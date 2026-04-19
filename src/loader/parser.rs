use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use iqos::{DeviceCapability, Iqos, IqosBle};
use rustyline::error::ReadlineError;
use rustyline::{Config, DefaultEditor, Editor};
use tokio::sync::Mutex;

use crate::loader::cmds::command::{CommandFn, CommandRegistry};
use crate::loader::iqoshelper::IqosHelper;

pub struct IQOSConsole {
    commands: CommandRegistry,
    pub iqos: Arc<Mutex<Iqos<IqosBle>>>,
}

impl IQOSConsole {
    pub fn new(iqos: Iqos<IqosBle>) -> Self {
        Self {
            commands: HashMap::new(),
            iqos: Arc::new(Mutex::new(iqos)),
        }
    }

    pub fn register_command(&mut self, name: &str, command: CommandFn) {
        self.commands.insert(name.to_string(), command);
    }

    async fn execute_command(&self, command: &str, args: Vec<String>) -> Result<()> {
        if let Some(cmd) = self.commands.get(command) {
            cmd(self.iqos.clone(), args).await
        } else {
            println!("Unknown command: {command}");
            Ok(())
        }
    }

    pub async fn run(&self) -> Result<()> {
        println!("IQOS Command Console v0.1.0");
        println!("Type 'help' to display available commands, 'exit' to quit");

        let config = Config::builder().build();
        let mut rl = Editor::<IqosHelper, rustyline::history::DefaultHistory>::with_config(config)?;
        rl.set_helper(Some(IqosHelper::new()));

        if rl.load_history("history.txt").is_err() {
            println!("No history file found");
        }

        loop {
            match tokio::task::block_in_place(|| rl.readline("iqos> ")) {
                Ok(line) => {
                    let _ = rl.add_history_entry(&line);
                    let args: Vec<String> = line.split_whitespace().map(str::to_string).collect();
                    if args.is_empty() {
                        continue;
                    }
                    let cmd = args[0].to_ascii_lowercase();
                    if cmd == "exit" || cmd == "quit" {
                        println!("Goodbye!");
                        break;
                    }
                    if let Err(e) = self.execute_command(&cmd, args).await {
                        println!("Error: {e}");
                    }
                }
                Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => break,
                Err(e) => {
                    println!("Error: {e:?}");
                    break;
                }
            }
        }

        rl.save_history("history.txt")?;
        Ok(())
    }
}

pub async fn run_console(iqos: Iqos<IqosBle>) -> Result<()> {
    let mut console = IQOSConsole::new(iqos);
    register_all_commands(&mut console);
    console.run().await
}

fn register_all_commands(console: &mut IQOSConsole) {
    register_builtin_commands(console);
    crate::loader::cmds::flexpuff::register_command(console);
    crate::loader::cmds::flexbattery::register_command(console);
    crate::loader::cmds::brightness::register_command(console);
    crate::loader::cmds::vibration::register_command(console);
    crate::loader::cmds::autostart::register_command(console);
    crate::loader::cmds::smartgesture::register_command(console);
    crate::loader::cmds::diagnosis::register_command(console);
}

fn register_builtin_commands(console: &mut IQOSConsole) {
    console.register_command(
        "help",
        Box::new(|iqos, _| {
            Box::pin(async move {
                let iqos = iqos.lock().await;
                let model = iqos.transport().model();
                println!("Available commands:");
                println!("  battery            Display battery level");
                println!("  lock | unlock      Lock or unlock the device");
                println!("  findmyiqos         Activate find-my-device vibration");
                println!("  autostart [on|off] Configure auto-start");
                println!("  diagnosis          Retrieve telemetry data");
                if model.is_iluma_family() {
                    println!("\nILUMA commands:");
                    println!("  brightness [high|low]                     Set brightness");
                    println!("  smartgesture [enable|disable]             Configure SmartGesture");
                    println!("  flexpuff [enable|disable|status]          Configure FlexPuff");
                    println!("  vibration [heating|starting|terminated|puffend] [on|off] ...");
                    if model.supports(DeviceCapability::FlexBattery) {
                        println!("  flexbattery [performance|eco|pause on|off]");
                    }
                }
                println!("\n  info               Device information");
                println!("  help               This help");
                println!("  quit | exit        Exit");
                Ok(())
            })
        }),
    );

    console.register_command(
        "battery",
        Box::new(|iqos, _| {
            Box::pin(async move {
                let iqos = iqos.lock().await;
                let level = iqos.transport().read_battery_level().await?;
                println!("Battery: {level}%");
                Ok(())
            })
        }),
    );

    console.register_command(
        "info",
        Box::new(|iqos, _| {
            Box::pin(async move {
                let iqos = iqos.lock().await;
                let model = iqos.transport().model();
                let info = iqos.transport().device_info();
                println!("\nModel:        {:?}", model);
                println!(
                    "Serial:       {}",
                    info.serial_number.as_deref().unwrap_or("N/A")
                );
                println!(
                    "Software:     {}",
                    info.software_revision.as_deref().unwrap_or("N/A")
                );
                println!(
                    "Manufacturer: {}",
                    info.manufacturer_name.as_deref().unwrap_or("N/A")
                );
                println!();
                Ok(())
            })
        }),
    );

    console.register_command(
        "lock",
        Box::new(|iqos, _| {
            Box::pin(async move {
                let iqos = iqos.lock().await;
                iqos.lock(iqos.transport().model()).await?;
                println!("Device locked");
                Ok(())
            })
        }),
    );

    console.register_command(
        "unlock",
        Box::new(|iqos, _| {
            Box::pin(async move {
                let iqos = iqos.lock().await;
                iqos.unlock(iqos.transport().model()).await?;
                println!("Device unlocked");
                Ok(())
            })
        }),
    );

    console.register_command(
        "findmyiqos",
        Box::new(|iqos, _| {
            Box::pin(async move {
                println!("Starting Find My IQOS...");

                {
                    let iqos = iqos.lock().await;
                    iqos.find_my_iqos_start().await?;
                }

                let prompt_result = tokio::task::block_in_place(|| -> Result<()> {
                    let mut rl = DefaultEditor::new()?;
                    let _ = rl.readline("Press <Enter> to stop");
                    Ok(())
                });

                let stop_result = {
                    let iqos = iqos.lock().await;
                    iqos.find_my_iqos_stop().await
                };

                prompt_result?;
                stop_result?;
                println!("Stopped.");
                Ok(())
            })
        }),
    );
}
