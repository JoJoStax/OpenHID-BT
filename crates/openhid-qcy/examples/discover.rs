use openhid_qcy::transport::enumerate_bluez_qcy_devices;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let devices = enumerate_bluez_qcy_devices().await?;
    println!("Discovered {} QCY device(s):", devices.len());
    for dev in devices {
        println!(
            " - {} [model={}] ({}) form_factor={} connected={} paired={} battery={:?} anc={}",
            dev.model.marketing_name,
            dev.model.model_id,
            dev.address,
            dev.model.form_factor,
            dev.connected,
            dev.paired,
            dev.battery_percentage,
            dev.model.capabilities.has_anc
        );
    }
    Ok(())
}
