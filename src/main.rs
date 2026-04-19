use std::collections::HashSet;
use std::io::{self, Write};

use anyhow::{Context as _, Result};
use btleplug::api::{Central, CentralEvent, Manager as _, Peripheral as _, ScanFilter};
use btleplug::platform::{Adapter, Manager, PeripheralId};
use colored::Colorize;
use futures::stream::StreamExt;
use iqos::{Iqos, IqosBle};

mod loader;

use loader::run_console;

async fn get_central(manager: &Manager) -> Result<Adapter> {
    manager
        .adapters()
        .await?
        .into_iter()
        .next()
        .context("No Bluetooth adapters found")
}

async fn prompt_for_connection(name: &str, addr: &PeripheralId) -> Result<bool> {
    let prompt = format!("Connect to {name} ({addr})? [y/n]: ");

    tokio::task::spawn_blocking(move || loop {
        print!("{prompt}");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        match input.trim() {
            answer if answer.eq_ignore_ascii_case("y") => return Ok(true),
            answer if answer.eq_ignore_ascii_case("n") => return Ok(false),
            _ => {}
        }
    })
    .await?
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
async fn main() -> Result<()> {
    let manager = Manager::new().await?;
    println!("{}", IQOS_CLI_ASCII_ART.blue());

    let central = get_central(&manager).await?;
    let central_state = central.adapter_state().await?;
    println!("CentralState: {:?}", central_state);

    let mut events = central.events().await?;
    let mut ignore_devices = HashSet::<PeripheralId>::new();

    println!("Scanning for IQOS devices...");
    central.start_scan(ScanFilter::default()).await?;

    while let Some(event) = events.next().await {
        match event {
            CentralEvent::DeviceDiscovered(addr) => {
                let peripheral = central.peripheral(&addr).await?;
                let properties = peripheral.properties().await?;
                let name = properties.and_then(|p| p.local_name).unwrap_or_default();

                if name.contains("IQOS") && !ignore_devices.contains(&addr) {
                    println!("Found IQOS: {name} ({addr})");

                    if prompt_for_connection(&name, &addr).await? {
                        println!("Connecting...");
                        let ble = IqosBle::connect_and_discover(peripheral).await?;
                        let iqos = Iqos::new(ble);
                        central.stop_scan().await?;
                        run_console(iqos).await?;
                        return Ok(());
                    }

                    ignore_devices.insert(addr.clone());
                    println!("Scanning for other devices...");
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
