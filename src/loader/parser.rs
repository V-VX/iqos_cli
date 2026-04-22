use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use iqos::{Iqos, IqosBle};
use rustyline::error::ReadlineError;
use rustyline::{Config, Editor};
use tokio::sync::Mutex;

use crate::loader::cmds::command::{CommandFn, CommandRegistry};
use crate::loader::iqoshelper::IqosHelper;

pub struct IQOSConsole {
    commands: CommandRegistry,
    iqos: Arc<Mutex<Iqos<IqosBle>>>,
}

impl IQOSConsole {
    pub fn new(iqos: Iqos<IqosBle>) -> Self {
        Self {
            commands: HashMap::with_capacity(16),
            iqos: Arc::new(Mutex::new(iqos)),
        }
    }

    pub fn register_command(&mut self, name: &str, command: CommandFn) {
        self.commands.insert(name.to_string(), command);
    }

    pub async fn execute_command(&self, command: &str, args: Vec<String>) -> Result<bool> {
        if let Some(cmd) = self.commands.get(command) {
            cmd(self.iqos.clone(), args).await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub async fn run(&self) -> Result<()> {
        println!("IQOS Command Console v0.1.0");
        println!("Type 'help' to display available commands, 'exit' to quit");

        let config = Config::builder().build();
        let mut rl = Editor::<IqosHelper, rustyline::history::DefaultHistory>::with_config(config)?;
        rl.set_helper(Some(IqosHelper::new()));

        let history_path = history_file();
        if rl.load_history(&history_path).is_err() {
            println!("No history file found");
        }

        loop {
            match tokio::task::block_in_place(|| rl.readline("iqos> ")) {
                Ok(line) => {
                    let args: Vec<String> = line.split_whitespace().map(str::to_string).collect();
                    if args.is_empty() {
                        continue;
                    }
                    if let Err(e) = rl.add_history_entry(&line) {
                        eprintln!("Warning: could not save history entry: {e}");
                    }
                    let cmd = args[0].to_ascii_lowercase();
                    if cmd == "exit" || cmd == "quit" {
                        println!("Goodbye!");
                        break;
                    }
                    match self.execute_command(&cmd, args).await {
                        Ok(true) => {}
                        Ok(false) => println!("Unknown command: {cmd}"),
                        Err(e) => eprintln!("Error: {e}"),
                    }
                }
                Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => break,
                Err(e) => {
                    eprintln!("Error: {e:?}");
                    break;
                }
            }
        }

        if let Err(e) = rl.save_history(&history_path) {
            eprintln!("Warning: could not save history: {e}");
        }
        Ok(())
    }
}

pub async fn run_console(iqos: Iqos<IqosBle>) -> Result<()> {
    let mut console = IQOSConsole::new(iqos);
    register_all_commands(&mut console);
    console.run().await
}

pub async fn run_registered_command(
    iqos: Iqos<IqosBle>,
    command: &str,
    args: Vec<String>,
) -> Result<()> {
    let mut console = IQOSConsole::new(iqos);
    register_all_commands(&mut console);
    if console.execute_command(command, args).await? {
        Ok(())
    } else {
        anyhow::bail!("Unknown command: {command}");
    }
}

fn register_all_commands(console: &mut IQOSConsole) {
    crate::loader::cmds::help::register_command(console);
    crate::loader::cmds::battery::register_command(console);
    crate::loader::cmds::info::register_command(console);
    crate::loader::cmds::lock::register_command(console);
    crate::loader::cmds::unlock::register_command(console);
    crate::loader::cmds::findmyiqos::register_command(console);
    crate::loader::cmds::flexpuff::register_command(console);
    crate::loader::cmds::flexbattery::register_command(console);
    crate::loader::cmds::brightness::register_command(console);
    crate::loader::cmds::vibration::register_command(console);
    crate::loader::cmds::autostart::register_command(console);
    crate::loader::cmds::smartgesture::register_command(console);
    crate::loader::cmds::diagnosis::register_command(console);
}

fn history_file() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join(".iqos_history")
}
