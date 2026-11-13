//! Cross-platform USB/2.4GHz HID transport for Razer mice via `async-hid`.

use std::collections::BTreeMap;
use std::time::Duration;

use async_hid::{AsyncHidFeatureHandle, Device, DeviceId, HidBackend};
use futures_lite::StreamExt;
use thiserror::Error;
use tokio::time::sleep;

use super::catalog::{RAZER_USB_VENDOR_ID, RazerMouseDescriptor, find_razer_mouse, resolve_mouse};
use super::report::{
    MouseLedEffect, PollingRate, RAZER_REPORT_LEN, RazerMouseProtocolError, RazerReport,
    RazerReportStatus, ScrollMode, cmd_get_battery, cmd_get_charging, cmd_get_dpi_xy,
    cmd_get_firmware_version, cmd_get_polling_rate, cmd_get_polling_rate_ext,
    cmd_get_scroll_mode, cmd_get_serial, cmd_set_dpi_xy, cmd_set_idle_time,
    cmd_set_led_brightness, cmd_set_low_battery_threshold, cmd_set_matrix_led,
    cmd_set_polling_rate, cmd_set_polling_rate_ext, cmd_set_scroll_mode, parse_battery_percentage,
    parse_dpi_xy,
};

/// Errors raised by Razer mouse HID transport.
#[derive(Debug, Error)]
pub enum RazerMouseError {
    #[error("HID transport error: {0}")]
    Hid(String),
    #[error("protocol error: {0}")]
    Protocol(#[from] RazerMouseProtocolError),
    #[error("device did not respond to feature report on any HID interface")]
    NoWorkingInterface,
    #[error("device timed out waiting for non-busy response")]
    BusyTimeout,
}

/// Discovered Razer USB/2.4GHz mouse with its candidate HID interfaces.
#[derive(Debug, Clone)]
pub struct DiscoveredRazerMouse {
    pub product_id: u16,
    pub os_name: String,
    pub serial_number: Option<String>,
    pub model: RazerMouseDescriptor,
    pub usage_page: u16,
    pub usage_id: u16,
    pub primary_id: DeviceId,
    pub candidate_ids: Vec<DeviceId>,
}

/// Enumerate connected Razer USB / 2.4GHz HyperSpeed mice via `async-hid`.
pub async fn enumerate_razer_mice() -> Result<Vec<DiscoveredRazerMouse>, RazerMouseError> {
    let backend = HidBackend::default();
    let mut stream = backend
        .enumerate()
        .await
        .map_err(|e| RazerMouseError::Hid(e.to_string()))?;

    let mut raw_nodes: Vec<Device> = Vec::new();
    while let Some(dev) = stream.next().await {
        if dev.vendor_id != RAZER_USB_VENDOR_ID {
            continue;
        }
        let known_mouse = find_razer_mouse(dev.product_id).is_some();
        let looks_like_mouse = dev.usage_page == 0x0001 && dev.usage_id == 0x0002;
        if known_mouse || looks_like_mouse {
            raw_nodes.push(dev);
        }
    }

    // Group interfaces belonging to the same physical mouse by (product_id, serial).
    let mut grouped: BTreeMap<(u16, String), Vec<Device>> = BTreeMap::new();
    for dev in raw_nodes {
        let key = (
            dev.product_id,
            dev.serial_number.clone().unwrap_or_default(),
        );
        grouped.entry(key).or_default().push(dev);
    }

    let mut mice = Vec::with_capacity(grouped.len());
    for ((pid, _), mut nodes) in grouped {
        // Prefer vendor-defined collections first, then standard mouse collection.
        nodes.sort_by_key(|d| {
            if d.usage_page >= 0xff00 {
                0u8
            } else if d.usage_page == 0x0001 && d.usage_id == 0x0002 {
                1u8
            } else {
                2u8
            }
        });

        let primary = &nodes[0];
        let model = resolve_mouse(pid, &primary.name);
        let candidate_ids = nodes.iter().map(|d| d.id.clone()).collect();
        mice.push(DiscoveredRazerMouse {
            product_id: pid,
            os_name: primary.name.clone(),
            serial_number: primary.serial_number.clone(),
            model,
            usage_page: primary.usage_page,
            usage_id: primary.usage_id,
            primary_id: primary.id.clone(),
            candidate_ids,
        });
    }

    Ok(mice)
}

/// Async client for sending 90-byte Razer HID feature reports to a mouse.
pub struct RazerMouseClient {
    pub device: DiscoveredRazerMouse,
    tx_id: u8,
}

impl RazerMouseClient {
    #[must_use]
    pub const fn new(device: DiscoveredRazerMouse) -> Self {
        let tx_id = device.model.capabilities.transaction_id;
        Self { device, tx_id }
    }

    /// Send a 90-byte Razer command and read back the response report.
    pub async fn send_report(
        &self,
        report: &RazerReport,
    ) -> Result<RazerReport, RazerMouseError> {
        let backend = HidBackend::default();
        let tx_candidates = [self.tx_id, 0x3f, 0xff];
        let mut last_err: Option<RazerMouseError> = None;

        for id in &self.device.candidate_ids {
            let Ok(mut devs) = backend.query_devices(id).await else {
                continue;
            };
            let Some(dev) = devs.next() else {
                continue;
            };
            let Ok(mut handle) = dev.open_feature_handle().await else {
                continue;
            };

            for &tx in &tx_candidates {
                let mut req = report.clone();
                req.transaction_id = tx;
                let payload = req.encode_with_report_id();

                if let Err(e) = handle.write_feature_report(&payload).await {
                    last_err = Some(RazerMouseError::Hid(e.to_string()));
                    continue;
                }

                // Razer firmware typically needs a brief settle delay before GET_FEATURE.
                for _attempt in 0..5 {
                    sleep(Duration::from_millis(12)).await;
                    let mut resp_buf = [0u8; RAZER_REPORT_LEN + 1];
                    match handle.read_feature_report(&mut resp_buf).await {
                        Ok(n) if n >= RAZER_REPORT_LEN => {
                            let decoded = match RazerReport::decode(&resp_buf[..n]) {
                                Ok(r) => r,
                                Err(e) => {
                                    last_err = Some(RazerMouseError::Protocol(e));
                                    break;
                                }
                            };
                            match decoded.parsed_status() {
                                Some(RazerReportStatus::Busy) => {}
                                Some(RazerReportStatus::Successful) => return Ok(decoded),
                                other => {
                                    return Err(RazerMouseError::Protocol(
                                        RazerMouseProtocolError::DeviceStatus(other),
                                    ));
                                }
                            }
                        }
                        Ok(n) => {
                            last_err = Some(RazerMouseError::Protocol(
                                RazerMouseProtocolError::InvalidReportLength(n),
                            ));
                            break;
                        }
                        Err(e) => {
                            last_err = Some(RazerMouseError::Hid(e.to_string()));
                            break;
                        }
                    }
                }
            }
        }

        Err(last_err.unwrap_or(RazerMouseError::NoWorkingInterface))
    }

    pub async fn get_firmware_version(&self) -> Result<(u8, u8), RazerMouseError> {
        let resp = self
            .send_report(&cmd_get_firmware_version(self.tx_id))
            .await?;
        Ok((resp.arguments[0], resp.arguments[1]))
    }

    pub async fn get_serial(&self) -> Result<String, RazerMouseError> {
        let resp = self.send_report(&cmd_get_serial(self.tx_id)).await?;
        let end = resp.arguments[..22]
            .iter()
            .position(|&b| b == 0)
            .unwrap_or(22);
        Ok(String::from_utf8_lossy(&resp.arguments[..end]).to_string())
    }

    pub async fn get_dpi(&self) -> Result<(u16, u16), RazerMouseError> {
        let resp = self.send_report(&cmd_get_dpi_xy(self.tx_id)).await?;
        Ok(parse_dpi_xy(&resp))
    }

    pub async fn set_dpi(&self, dpi_x: u16, dpi_y: u16) -> Result<(), RazerMouseError> {
        let req = cmd_set_dpi_xy(self.tx_id, dpi_x, dpi_y)?;
        self.send_report(&req).await?;
        Ok(())
    }

    pub async fn get_polling_rate(&self) -> Result<PollingRate, RazerMouseError> {
        if self.device.model.capabilities.supports_hyperpolling
            && let Ok(resp) = self
                .send_report(&cmd_get_polling_rate_ext(self.tx_id))
                .await
            && let Some(rate) = PollingRate::from_extended_code(resp.arguments[1])
        {
            return Ok(rate);
        }
        let resp = self.send_report(&cmd_get_polling_rate(self.tx_id)).await?;
        PollingRate::from_standard_code(resp.arguments[0]).ok_or_else(|| {
            RazerMouseError::Protocol(RazerMouseProtocolError::UnsupportedPollingRate(u16::from(
                resp.arguments[0],
            )))
        })
    }

    pub async fn set_polling_rate(&self, rate: PollingRate) -> Result<(), RazerMouseError> {
        if self.device.model.capabilities.supports_hyperpolling
            || rate.standard_code().is_none()
        {
            let req = cmd_set_polling_rate_ext(self.tx_id, rate);
            if self.send_report(&req).await.is_ok() {
                return Ok(());
            }
        }
        let req = cmd_set_polling_rate(self.tx_id, rate)?;
        self.send_report(&req).await?;
        Ok(())
    }

    pub async fn get_battery(&self) -> Result<u8, RazerMouseError> {
        let resp = self.send_report(&cmd_get_battery(self.tx_id)).await?;
        Ok(parse_battery_percentage(&resp))
    }

    pub async fn is_charging(&self) -> Result<bool, RazerMouseError> {
        let resp = self.send_report(&cmd_get_charging(self.tx_id)).await?;
        Ok(resp.arguments[1] == 0x01)
    }

    pub async fn set_idle_time(&self, seconds: u16) -> Result<(), RazerMouseError> {
        let req = cmd_set_idle_time(self.tx_id, seconds);
        self.send_report(&req).await?;
        Ok(())
    }

    pub async fn set_low_battery_threshold(&self, percent: u8) -> Result<(), RazerMouseError> {
        let req = cmd_set_low_battery_threshold(self.tx_id, percent);
        self.send_report(&req).await?;
        Ok(())
    }

    pub async fn set_led(&self, led_id: u8, effect: MouseLedEffect) -> Result<(), RazerMouseError> {
        let req = cmd_set_matrix_led(self.tx_id, led_id, effect);
        self.send_report(&req).await?;
        Ok(())
    }

    pub async fn set_brightness(&self, led_id: u8, brightness: u8) -> Result<(), RazerMouseError> {
        let req = cmd_set_led_brightness(self.tx_id, led_id, brightness);
        self.send_report(&req).await?;
        Ok(())
    }

    pub async fn get_scroll_mode(&self) -> Result<ScrollMode, RazerMouseError> {
        let resp = self.send_report(&cmd_get_scroll_mode(self.tx_id)).await?;
        Ok(if resp.arguments[1] == ScrollMode::FreeSpin as u8 {
            ScrollMode::FreeSpin
        } else {
            ScrollMode::Tactile
        })
    }

    pub async fn set_scroll_mode(&self, mode: ScrollMode) -> Result<(), RazerMouseError> {
        let req = cmd_set_scroll_mode(self.tx_id, mode);
        self.send_report(&req).await?;
        Ok(())
    }
}
