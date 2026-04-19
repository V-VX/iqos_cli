use std::io::Write;

use btleplug::api::{Central, CentralEvent, Manager as _, Peripheral as _, ScanFilter};
use btleplug::platform::{Adapter, Manager, PeripheralId};
use colored::Colorize;
use futures::stream::StreamExt;
use iqos::{Iqos, IqosBle};

mod loader;

use loader::run_console;

async fn get_central(manager: &Manager) -> Adapter {
    manager.adapters().await.unwrap().into_iter().next().unwrap()
}

const IQOS_CLI_ASCII_ART: &str = r"

 ██╗  ██████╗   ██████╗  ███████╗      ██████╗ ██╗      ██╗
 ██║ ██╔═══██╗ ██╔═══██╗ ██╔════╝     ██╔════╝ ██║      ██║
 ██║ ██║   ██║ ██║   ██║ ███████╗     ██║      ██║      ██║
 ██║ ██║▄▄ ██║ ██║   ██║ ╚════██║     ██║      ██║      ██║
 ██║ ╚██████╔╝ ╚██████╔╝ ███████║     ╚██████╗ ███████╗ ██║
 ╚═╝  ╚══▀▀═╝   ╚═════╝  ╚══════╝      ╚═════╝ ╚══════╝ ╚═╝

";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let manager = Manager::new().await.unwrap();
    println!("{}", IQOS_CLI_ASCII_ART.blue());

    let central = get_central(&manager).await;
    let central_state = central.adapter_state().await.unwrap();
    println!("CentralState: {:?}", central_state);

    let mut events = central.events().await?;
    let mut ignore_devices: Vec<PeripheralId> = vec![];

    println!("Scanning for IQOS devices...");
    central.start_scan(ScanFilter::default()).await?;

    while let Some(event) = events.next().await {
        match event {
            CentralEvent::DeviceDiscovered(addr) => {
                let peripheral = central.peripheral(&addr).await?;
                let properties = peripheral.properties().await?;
                let name = properties
                    .and_then(|p| p.local_name)
                    .unwrap_or_default();

                if name.contains("IQOS") && !ignore_devices.contains(&addr) {
                    println!("Found IQOS: {name} ({addr})");

                    loop {
                        print!("Connect to {name} ({addr})? [y/n]: ");
                        let _ = std::io::stdout().flush();
                        let mut input = String::new();
                        std::io::stdin().read_line(&mut input)?;

                        match input.trim().to_lowercase().as_str() {
                            "y" => {
                                println!("Connecting...");
                                let ble = IqosBle::connect_and_discover(peripheral).await?;
                                let iqos = Iqos::new(ble);
                                central.stop_scan().await?;
                                run_console(iqos).await?;
                                return Ok(());
                            }
                            "n" => {
                                ignore_devices.push(addr.clone());
                                println!("Scanning for other devices...");
                                break;
                            }
                            _ => {}
                        }
                    }
                }
            }
            CentralEvent::StateUpdate(state) => println!("State Update: {:?}", state),
            CentralEvent::DeviceConnected(id) => println!("Device Connected: {id}"),
            CentralEvent::DeviceDisconnected(id) => println!("Device Disconnected: {id}"),
            _ => {}
        }
    }

    Ok(())
}
