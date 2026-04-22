use std::sync::Arc;

use anyhow::Result;
use iqos::{Iqos, IqosBle};
use tokio::sync::Mutex;

use crate::cli::print_version;
use crate::loader::parser::{invalid_arguments, IQOSConsole};

pub fn register_command(console: &mut IQOSConsole) {
    console.register_command(
        "version",
        Box::new(|iqos, args| Box::pin(async move { execute(iqos, args).await })),
    );
}

async fn execute(_iqos: Arc<Mutex<Iqos<IqosBle>>>, args: Vec<String>) -> Result<()> {
    if args.len() != 1 {
        return Err(invalid_arguments("Usage: version"));
    }

    print_version();
    Ok(())
}
