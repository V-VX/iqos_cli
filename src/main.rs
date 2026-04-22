use std::collections::HashSet;
use std::io::{self, Write};
use std::time::Duration;

use anyhow::{anyhow, Context as _, Result};
use btleplug::api::{
    Central, CentralEvent, Manager as _, Peripheral as _, PeripheralProperties, ScanFilter,
};
use btleplug::platform::{Adapter, Manager, Peripheral, PeripheralId};
use clap::Parser;
use colored::Colorize;
use futures::stream::StreamExt;
use iqos::{DeviceModel, Iqos, IqosBle};

mod cli;
mod config;
mod loader;
mod model_selector;

use cli::{normalize_global_options, scan_timeout, Cli, OneShotCommand};
use config::{print_saved_devices, validate_device_label, AppConfig, ConnectedDevice};
use loader::{run_console_with_device, run_registered_command};
use model_selector::parse_device_model;

const EXIT_CONNECTION_FAILED: i32 = 1;
const EXIT_INVALID_ARGUMENTS: i32 = 2;
const EXIT_DEVICE_COMMAND_FAILED: i32 = 3;
const EXIT_LABEL_NOT_FOUND: i32 = 4;

#[derive(Debug)]
struct ExitError {
    code: i32,
    error: anyhow::Error,
}

impl ExitError {
    fn new(code: i32, error: impl Into<anyhow::Error>) -> Self {
        Self {
            code,
            error: error.into(),
        }
    }
}

#[derive(Debug, Clone)]
enum ScanTarget {
    Model(DeviceModel),
    Address {
        label: Option<String>,
        address: String,
        cached_serial: Option<String>,
    },
}

#[derive(Debug)]
struct DiscoveredDevice {
    address: String,
    local_name: Option<String>,
}

#[derive(Debug)]
struct ResolvedTarget {
    config: AppConfig,
    target: ScanTarget,
    should_save_memory: bool,
}

async fn get_central(manager: &Manager) -> Result<Adapter> {
    manager
        .adapters()
        .await?
        .into_iter()
        .next()
        .context("No Bluetooth adapters found")
}

async fn prompt_for_connection(name: &str, addr: &str) -> Result<bool> {
    let prompt = format!("Connect to {name} ({addr})? [y/n]: ");

    tokio::task::spawn_blocking(move || loop {
        print!("{prompt}");
        io::stdout().flush()?;

        let mut input = String::new();
        let n = io::stdin().read_line(&mut input)?;
        if n == 0 {
            return Ok(false);
        }

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
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let exit_code = if cli::should_use_cli(&args) {
        run_cli(args).await
    } else {
        match run_interactive().await {
            Ok(()) => 0,
            Err(error) => {
                eprintln!("Error: {error:#}");
                EXIT_CONNECTION_FAILED
            }
        }
    };

    std::process::exit(exit_code);
}

async fn run_cli(mut args: Vec<String>) -> i32 {
    if let Some(program) = args.first_mut() {
        *program = "iqos".to_string();
    }
    let args = normalize_global_options(args);

    let cli = match Cli::try_parse_from(args) {
        Ok(cli) => cli,
        Err(error) => {
            let code = error.exit_code();
            let _ = error.print();
            return code;
        }
    };

    let Some(command) = cli.command else {
        return match run_auto_connected_console(cli.model, scan_timeout(cli.timeout)).await {
            Ok(()) => 0,
            Err(error) => {
                eprintln!("Error: {:#}", error.error);
                error.code
            }
        };
    };

    match run_one_shot(
        cli.model,
        scan_timeout(cli.timeout),
        command.into_one_shot(),
    )
    .await
    {
        Ok(()) => 0,
        Err(error) => {
            eprintln!("Error: {:#}", error.error);
            error.code
        }
    }
}

async fn run_auto_connected_console(
    model_arg: Option<String>,
    timeout: Duration,
) -> std::result::Result<(), ExitError> {
    let ResolvedTarget {
        mut config,
        target,
        should_save_memory,
    } = load_config_and_resolve_target(model_arg.as_deref(), true)?;
    let (iqos, device) = connect_target(&target, timeout).await?;

    apply_connection_memory(&mut config, &target, &device);
    save_connection_memory(&config, &target, should_save_memory, true)?;

    run_console_with_device(Iqos::new(iqos), device)
        .await
        .map_err(|error| ExitError::new(EXIT_DEVICE_COMMAND_FAILED, error))
}

async fn run_one_shot(
    model_arg: Option<String>,
    timeout: Duration,
    command: OneShotCommand,
) -> std::result::Result<(), ExitError> {
    match command {
        OneShotCommand::DeviceList => {
            let config =
                AppConfig::load().map_err(|error| ExitError::new(EXIT_CONNECTION_FAILED, error))?;
            print_saved_devices(&config);
            Ok(())
        }
        OneShotCommand::DeviceRemove { label } => {
            let mut config =
                AppConfig::load().map_err(|error| ExitError::new(EXIT_CONNECTION_FAILED, error))?;
            if config.remove_device(&label) {
                config
                    .save()
                    .map_err(|error| ExitError::new(EXIT_CONNECTION_FAILED, error))?;
                println!("Removed device label: {label}");
                Ok(())
            } else {
                Err(ExitError::new(
                    EXIT_LABEL_NOT_FOUND,
                    anyhow!("Device label not found: {label}"),
                ))
            }
        }
        OneShotCommand::DeviceSave { label } => {
            validate_device_label(&label)
                .map_err(|error| ExitError::new(EXIT_INVALID_ARGUMENTS, error))?;
            let ResolvedTarget {
                mut config, target, ..
            } = load_config_and_resolve_target(model_arg.as_deref(), false)?;
            let (iqos, device) = connect_target(&target, timeout).await?;
            apply_connection_memory(&mut config, &target, &device);
            config
                .save_device(label.clone(), &device)
                .map_err(|error| ExitError::new(EXIT_INVALID_ARGUMENTS, error))?;
            config
                .save()
                .map_err(|error| ExitError::new(EXIT_CONNECTION_FAILED, error))?;
            drop(iqos);
            println!("Saved device label: {label}");
            Ok(())
        }
        OneShotCommand::Registered { name, args } => {
            let ResolvedTarget {
                config: mut command_config,
                target,
                should_save_memory,
            } = load_config_and_resolve_target(model_arg.as_deref(), true)?;
            let (iqos, device) = connect_target(&target, timeout).await?;
            apply_connection_memory(&mut command_config, &target, &device);
            save_connection_memory(&command_config, &target, should_save_memory, true)?;

            run_registered_command(Iqos::new(iqos), name, args)
                .await
                .map_err(|error| ExitError::new(classify_command_error(&error), error))
        }
    }
}

fn load_config_and_resolve_target(
    model_arg: Option<&str>,
    allow_model_config_failure: bool,
) -> std::result::Result<ResolvedTarget, ExitError> {
    if let Some(value) = model_arg {
        if let Some(model) = parse_device_model(value) {
            let (config, should_save_memory) = load_memory_config(allow_model_config_failure)?;
            return Ok(ResolvedTarget {
                config,
                target: ScanTarget::Model(model),
                should_save_memory,
            });
        }
    }

    let config =
        AppConfig::load().map_err(|error| ExitError::new(EXIT_CONNECTION_FAILED, error))?;
    let target = resolve_target(model_arg, &config)?;

    Ok(ResolvedTarget {
        config,
        target,
        should_save_memory: true,
    })
}

fn load_memory_config(allow_failure: bool) -> std::result::Result<(AppConfig, bool), ExitError> {
    match AppConfig::load() {
        Ok(config) => Ok((config, true)),
        Err(error) if allow_failure => {
            eprintln!("Warning: could not load device config: {error:#}");
            Ok((AppConfig::default(), false))
        }
        Err(error) => Err(ExitError::new(EXIT_CONNECTION_FAILED, error)),
    }
}

fn resolve_target(
    model_arg: Option<&str>,
    config: &AppConfig,
) -> std::result::Result<ScanTarget, ExitError> {
    match model_arg {
        Some(value) => {
            if let Some(model) = parse_device_model(value) {
                return Ok(ScanTarget::Model(model));
            }

            let saved = config.devices.get(value).ok_or_else(|| {
                ExitError::new(
                    EXIT_LABEL_NOT_FOUND,
                    anyhow!("Device label not found: {value}"),
                )
            })?;

            Ok(ScanTarget::Address {
                label: Some(value.to_string()),
                address: saved.address.clone(),
                cached_serial: saved.serial_number.clone(),
            })
        }
        None => {
            let default = config.default.as_ref().ok_or_else(|| {
                ExitError::new(
                    EXIT_LABEL_NOT_FOUND,
                    anyhow!("No default device stored. Connect interactively or use --model <DeviceModel>."),
                )
            })?;

            Ok(ScanTarget::Address {
                label: None,
                address: default.address.clone(),
                cached_serial: None,
            })
        }
    }
}

fn save_connection_memory(
    config: &AppConfig,
    target: &ScanTarget,
    should_save_memory: bool,
    allow_model_save_failure: bool,
) -> std::result::Result<(), ExitError> {
    if !should_save_memory {
        return Ok(());
    }

    match config.save() {
        Ok(()) => Ok(()),
        Err(error) if allow_model_save_failure && matches!(target, ScanTarget::Model(_)) => {
            eprintln!("Warning: could not save device config: {error:#}");
            Ok(())
        }
        Err(error) => Err(ExitError::new(EXIT_CONNECTION_FAILED, error)),
    }
}

async fn connect_target(
    target: &ScanTarget,
    timeout: Duration,
) -> std::result::Result<(IqosBle, ConnectedDevice), ExitError> {
    let manager = Manager::new()
        .await
        .map_err(|error| ExitError::new(EXIT_CONNECTION_FAILED, error))?;
    let central = get_central(&manager)
        .await
        .map_err(|error| ExitError::new(EXIT_CONNECTION_FAILED, error))?;

    let (peripheral, discovered) = find_matching_peripheral(&central, target, timeout).await?;

    let ble = IqosBle::connect_and_discover(peripheral)
        .await
        .map_err(|error| ExitError::new(EXIT_CONNECTION_FAILED, error))?;
    let device = connected_device(&ble, discovered);
    warn_serial_mismatch(target, &device);

    Ok((ble, device))
}

async fn find_matching_peripheral(
    central: &Adapter,
    target: &ScanTarget,
    timeout: Duration,
) -> std::result::Result<(Peripheral, DiscoveredDevice), ExitError> {
    let mut events = central
        .events()
        .await
        .map_err(|error| ExitError::new(EXIT_CONNECTION_FAILED, error))?;

    central
        .start_scan(ScanFilter::default())
        .await
        .map_err(|error| ExitError::new(EXIT_CONNECTION_FAILED, error))?;

    let result = tokio::time::timeout(timeout, async {
        while let Some(event) = events.next().await {
            let addr = match event {
                CentralEvent::DeviceDiscovered(addr) | CentralEvent::DeviceUpdated(addr) => addr,
                _ => continue,
            };

            let peripheral = central.peripheral(&addr).await?;
            let properties = peripheral.properties().await?;
            let discovered = discovered_device(&addr, properties.as_ref());

            if target_matches(target, &discovered) {
                return Ok::<_, anyhow::Error>(Some((peripheral, discovered)));
            }
        }

        Ok::<_, anyhow::Error>(None)
    })
    .await;

    if let Err(error) = central.stop_scan().await {
        eprintln!("Warning: could not stop BLE scan: {error}");
    }

    match result {
        Ok(Ok(Some(found))) => Ok(found),
        Ok(Ok(None)) => Err(ExitError::new(
            EXIT_CONNECTION_FAILED,
            anyhow!("BLE event stream ended before a matching IQOS device was found"),
        )),
        Ok(Err(error)) => Err(ExitError::new(EXIT_CONNECTION_FAILED, error)),
        Err(_) => Err(ExitError::new(
            EXIT_CONNECTION_FAILED,
            anyhow!(
                "Device not found before scan timeout: {}",
                describe_target(target)
            ),
        )),
    }
}

async fn run_interactive() -> Result<()> {
    let manager = Manager::new().await?;
    println!("{}", IQOS_CLI_ASCII_ART.blue());

    let central = get_central(&manager).await?;
    let central_state = central.adapter_state().await?;
    println!("CentralState: {:?}", central_state);

    let mut events = central.events().await?;
    let mut ignore_devices = HashSet::<String>::new();

    println!("Scanning for IQOS devices...");
    central.start_scan(ScanFilter::default()).await?;

    while let Some(event) = events.next().await {
        match event {
            CentralEvent::DeviceDiscovered(addr) => {
                let peripheral = central.peripheral(&addr).await?;
                let properties = peripheral.properties().await?;
                let discovered = discovered_device(&addr, properties.as_ref());
                let name = discovered.local_name.clone().unwrap_or_default();

                if name.contains("IQOS") && !ignore_devices.contains(&discovered.address) {
                    println!("Found IQOS: {name} ({})", discovered.address);

                    if prompt_for_connection(&name, &discovered.address).await? {
                        println!("Connecting...");
                        let ble = IqosBle::connect_and_discover(peripheral).await?;
                        let device = connected_device(&ble, discovered);
                        remember_connected_device(&device);
                        let iqos = Iqos::new(ble);
                        central.stop_scan().await?;
                        run_console_with_device(iqos, device).await?;
                        return Ok(());
                    }

                    ignore_devices.insert(discovered.address);
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

fn remember_connected_device(device: &ConnectedDevice) {
    match AppConfig::load() {
        Ok(mut config) => {
            config.update_default(device);
            if let Err(error) = config.save() {
                eprintln!("Warning: could not save device config: {error:#}");
            }
        }
        Err(error) => eprintln!("Warning: could not load device config: {error:#}"),
    }
}

fn discovered_device(
    addr: &PeripheralId,
    properties: Option<&PeripheralProperties>,
) -> DiscoveredDevice {
    let local_name = properties.and_then(|properties| properties.local_name.clone());
    let address = properties
        .map(|properties| properties.address.to_string())
        .filter(|address| address != "00:00:00:00:00:00")
        .unwrap_or_else(|| addr.to_string());

    DiscoveredDevice {
        address,
        local_name,
    }
}

fn connected_device(ble: &IqosBle, discovered: DiscoveredDevice) -> ConnectedDevice {
    ConnectedDevice {
        address: discovered.address,
        local_name: discovered.local_name,
        model: ble.model(),
        serial_number: ble.device_info().serial_number.clone(),
    }
}

fn target_matches(target: &ScanTarget, discovered: &DiscoveredDevice) -> bool {
    match target {
        ScanTarget::Model(model) => {
            discovered
                .local_name
                .as_deref()
                .map(DeviceModel::from_local_name)
                == Some(*model)
        }
        ScanTarget::Address { address, .. } => discovered.address.eq_ignore_ascii_case(address),
    }
}

fn apply_connection_memory(config: &mut AppConfig, target: &ScanTarget, device: &ConnectedDevice) {
    config.update_default(device);

    if let ScanTarget::Address {
        label: Some(label), ..
    } = target
    {
        config.update_saved_device_metadata(label, device);
    }
}

fn warn_serial_mismatch(target: &ScanTarget, device: &ConnectedDevice) {
    let ScanTarget::Address {
        label,
        cached_serial: Some(cached_serial),
        ..
    } = target
    else {
        return;
    };

    let Some(actual_serial) = &device.serial_number else {
        return;
    };

    if cached_serial != actual_serial {
        let target_name = label.as_deref().unwrap_or("default");
        eprintln!(
            "Warning: serial number mismatch for {target_name}: cached {cached_serial}, connected {actual_serial}"
        );
    }
}

fn classify_command_error(error: &anyhow::Error) -> i32 {
    let message = error.to_string();
    if message.starts_with("Usage:")
        || message.starts_with("Invalid option:")
        || message.starts_with("Invalid pause value:")
        || message.starts_with("Each flag requires")
        || message.starts_with("Unknown command:")
    {
        EXIT_INVALID_ARGUMENTS
    } else {
        EXIT_DEVICE_COMMAND_FAILED
    }
}

fn describe_target(target: &ScanTarget) -> String {
    match target {
        ScanTarget::Model(model) => format!("{model:?}"),
        ScanTarget::Address {
            label: Some(label),
            address,
            ..
        } => format!("{label} ({address})"),
        ScanTarget::Address { address, .. } => address.clone(),
    }
}
