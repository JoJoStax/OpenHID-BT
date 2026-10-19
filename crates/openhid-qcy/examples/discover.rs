use openhid_qcy::transport::enumerate_bluez_qcy_devices;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let devices = enumerate_bluez_qcy_devices().await?;
    println!("Discovered {} QCY device(s):", devices.len());
    for dev in devices {
        println!(" - {} ({}) connected={} paired={} battery={:?}", dev.name, dev.address, dev.connected, dev.paired, dev.battery_percentage);
    }
    Ok(())
}
