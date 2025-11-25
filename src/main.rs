use std::io::Write;
use btleplug::api::{
    Central, CentralEvent, Manager as _, Peripheral as _, ScanFilter,
};
use btleplug::platform::{Adapter, Manager, PeripheralId};
use colored::{ColoredString, Colorize};
use futures::stream::StreamExt;
use std::error::Error;

mod iqos;
mod loader;

use loader::run_console;
use iqos::{Iqos, IqosIluma, IqosBle};

async fn get_central(manager: &Manager) -> Adapter {
    let adapters = manager.adapters().await.unwrap();
    adapters.into_iter().nth(0).unwrap()
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
async fn main() -> Result<(), Box<dyn Error>> {
    let mut iqos_builder: iqos::IQOSBuilder;
    let manager = Manager::new().await.unwrap();
    println!("{}", IQOS_CLI_ASCII_ART.blue());

    // get the first bluetooth adapter
    let central = get_central(&manager).await;

    let central_state = central.adapter_state().await.unwrap();
    println!("CentralState: {:?}", central_state);

    let mut events = central.events().await?;
    let mut ignore_devices: Vec<PeripheralId> = vec![];

    println!("Scanning for IQOS devices...");
    // start scanning for devices
    central.start_scan(ScanFilter::default()).await?;

    while let Some(event) = events.next().await {
        match event {
            CentralEvent::DeviceDiscovered(addr) => {
                let peripheral = central.peripheral(&addr).await?;
                let properties = peripheral.properties().await?;
                let name = properties
                    .and_then(|p| p.local_name)
                    .map(|local_name| local_name.to_string())
                    .unwrap_or_default();

                if name.contains("IQOS") && !ignore_devices.contains(&addr) {
                    iqos_builder = iqos::IQOSBuilder::new(peripheral);

                    println!("Found IQOS: {name} ({addr})");
                    
                    loop {
                        print!("Connect to {name} ({addr})? [y/n]: ");
                        let _  = std::io::stdout().flush();
                        let mut input = String::new();
                        std::io::stdin().read_line(&mut input)?;

                        if input.trim().to_lowercase() == "y" {
                            println!("Connecting to IQOS device...");
                            iqos_builder.connect().await?;
                            println!("Connected!");
                            let _services = iqos_builder.discover_services().await?;
                            iqos_builder.initialize().await?;

                            let iqos = iqos_builder.build().await?;
                            central.stop_scan().await?;
                            run_console(iqos).await?;
                            return Ok(());
                        } else if input.trim().to_lowercase() == "n" {
                            ignore_devices.push(addr.clone());
                            println!("Scanning for other devices...");
                            break;
                        }
                        continue
                    }
                }
            }
            CentralEvent::StateUpdate(state) => {
                println!("State Update: {:?}", state);
            }
            CentralEvent::DeviceConnected(id) => {
                println!("Device Connected: {id}");
            }
            CentralEvent::DeviceDisconnected(id) => {
                println!("Device Disconnected: {id}");
            }
            _ => {}
        }
    }
    Ok(())
}