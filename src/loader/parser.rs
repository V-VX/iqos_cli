use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use rustyline::error::ReadlineError;
use rustyline::{Config, Editor, DefaultEditor};
use tokio::sync::Mutex;

use crate::iqos::IqosBle;
use crate::iqos::device::Iqos;
use crate::loader::cmds::command::{CommandFn, CommandRegistry};
use crate::loader::iqoshelper::IqosHelper;

/// The main console handler for the IQOS CLI
pub struct IQOSConsole {
    commands: Arc<Mutex<CommandRegistry>>,
    pub iqos: Arc<Mutex<IqosBle>>,
}

impl IQOSConsole {
    /// Create a new console instance
    pub fn new(iqos: IqosBle) -> Self {
        Self {
            commands: Arc::new(Mutex::new(HashMap::new())),
            iqos: Arc::new(Mutex::new(iqos)),
        }
    }
    
    /// Register a new command
    pub async fn register_command(&self, name: &str, command: CommandFn) {
        let mut commands = self.commands.lock().await;
        commands.insert(name.to_string(), command);
    }
    
    /// Register multiple commands at once
    pub async fn register_commands<F>(&self, register_fn: F) 
    where
        F: FnOnce(&mut CommandRegistry),
    {
        let mut commands = self.commands.lock().await;
        register_fn(&mut commands);
    }
    
    /// Execute a command
    pub async fn execute_command(&self, command: &str, args: Vec<String>) -> Result<()> {
        let commands = self.commands.lock().await;
        
        if let Some(cmd) = commands.get(command) {
            cmd(self.iqos.clone(), args).await
        } else {
            println!("Unknown command: {}", command);
            Ok(())
        }
    }
    
    /// List all registered commands
    pub async fn list_commands(&self) -> Vec<String> {
        let commands = self.commands.lock().await;
        commands.keys().cloned().collect()
    }
    
    /// Run the console interactive loop
    pub async fn run(&self) -> Result<()> {
        println!("IQOS Command Console v0.1.0");
        println!("Type 'help' to display available commands, 'exit' to quit");
        
        let config = Config::builder().build();
        let mut rl = Editor::<IqosHelper, rustyline::history::DefaultHistory>::with_config(config)?;
        let helper = IqosHelper::new();
        rl.set_helper(Some(helper));
        
        if rl.load_history("history.txt").is_err() {
            println!("No history file found");
        }
        
        loop {
            match rl.readline("iqos> ") {
                Ok(line_str) => {
                    let _ = rl.add_history_entry(&line_str);
                    
                    let args: Vec<String> = line_str.trim()
                        .split_whitespace()
                        .map(|s| s.to_string())
                        .collect();
    
                    if args.is_empty() {
                        continue;
                    }
                    
                    let cmd = args[0].to_lowercase();
                    
                    if cmd == "exit" || cmd == "quit" {
                        let mut iqos = self.iqos.lock().await;
                        // Use the Iqos trait method
                        let _ = Iqos::disconnect(&mut *iqos).await;
                        println!("Goodbye!");
                        break;
                    }
                    
                    if let Err(e) = self.execute_command(&cmd, args).await {
                        println!("Command execution error: {}", e);
                    }
                },
                Err(ReadlineError::Interrupted) => {
                    println!("Ctrl-C");
                    break;
                },
                Err(ReadlineError::Eof) => {
                    println!("Ctrl-D");
                    break;
                },
                Err(err) => {
                    println!("Error: {:?}", err);
                    break;
                }
            }
        }
        
        rl.save_history("history.txt")?;
        
        Ok(())
    }
}

/// Run the console application
pub async fn run_console(iqos: IqosBle) -> Result<()> {
    let console = IQOSConsole::new(iqos);
    
    // Register all commands
    register_all_commands(&console).await;
    
    console.run().await
}

/// Register all available commands
async fn register_all_commands(console: &IQOSConsole) {
    // Register built-in commands first
    register_builtin_commands(console).await;
    
    // Register commands from other modules
    crate::loader::cmds::flexpuff::register_command(console).await;
    crate::loader::cmds::flexbattery::register_command(console).await;
    crate::loader::cmds::brightness::register_command(console).await;
    crate::loader::cmds::vibration::register_command(console).await;
    crate::loader::cmds::autostart::register_command(console).await;
    crate::loader::cmds::smartgesture::register_command(console).await;
    crate::loader::cmds::diagnosis::register_command(console).await;
    
    // TODO: Register other command modules here as needed
}

/// Register built-in simple commands
async fn register_builtin_commands(console: &IQOSConsole) {
    // Register help command
    console.register_command("help", Box::new(|iqos, _| {
        Box::pin(async move {
            let iqos = iqos.lock().await;
            
            println!("Available commands:");
            
            // Common commands
            println!("  battery - Display battery status");
            println!("  lock | unlock - Lock or unlock the device");
            println!("  findmyiqos - Activate find my device feature");
            println!("  autostart [on|off] - Configure auto-start feature");
            
            // ILUMA model specific commands
            if iqos.is_iluma_or_higher() {
                println!("\nILUMA model specific commands:");
                println!("  brightness [high|low] - Set brightness");
                println!("  smartgesture [enable|disable] - Configure Smart Gesture feature");
                println!("  flexpuff [enable|disable] - Configure FlexPuff feature");
                println!("  vibration [option on|off]... - Configure vibration settings");
                println!("    Example: vibration charge on heating on puffend on");
                println!("    Options: charge, heating, starting, terminated, puffend");
            }
            
            println!("\nOther commands:");
            println!("  info - Display device status");
            println!("  help - Display this help message");
            println!("  quit | exit - Exit the program");
            
            Ok(())
        })
    })).await;
    
    // Register battery command
    console.register_command("battery", Box::new(|iqos, _| {
        Box::pin(async move {
            let mut iqos = iqos.lock().await;
            // Use the Iqos trait methods explicitly
            Iqos::reload_battery(&mut *iqos).await?;
            println!("Battery status: {}%", Iqos::battery_status(&*iqos));
            Ok(())
        })
    })).await;

    // Register info command
    console.register_command("info", Box::new(|iqos, _| {
        Box::pin(async move {
            let iqos = iqos.lock().await;
            println!("\n{}\n", iqos);
            Ok(())
        })
    })).await;
    
    // Register lock command
    console.register_command("lock", Box::new(|iqos, _| {
        Box::pin(async move {
            let iqos = iqos.lock().await;
            // Use the Iqos trait method explicitly
            Iqos::lock_device(&*iqos).await?;
            println!("Locked the IQOS");
            Ok(())
        })
    })).await;

    // Register unlock command
    console.register_command("unlock", Box::new(|iqos, _| {
        Box::pin(async move {
            let iqos = iqos.lock().await;
            // Use the Iqos trait method explicitly
            Iqos::unlock_device(&*iqos).await?;
            println!("Unlocked the IQOS");
            Ok(())
        })
    })).await;
    
    // Register findmyiqos command
    console.register_command("findmyiqos", Box::new(|iqos, _| {
        let mut rl = DefaultEditor::new().unwrap();
        Box::pin(async move {
            let iqos = iqos.lock().await;
            println!("Find My IQOS...");
            // Use the Iqos trait method explicitly
            Iqos::vibrate(&*iqos).await?;
            let input = rl.readline("Press <Enter> to stop vibration");
            
            match input {
                Ok(_) => {
                    // Use the Iqos trait method explicitly
                    Iqos::stop_vibrate(&*iqos).await?;
                    print!("Vibration stopped.\n");
                }
                Err(_) => {
                    println!("Vibration stopped.");
                }
            }

            Ok(())
        })
    })).await;
}
