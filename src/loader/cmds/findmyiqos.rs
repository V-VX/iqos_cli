use std::sync::Arc;

use anyhow::Result;
use iqos::{Iqos, IqosBle};
use rustyline::DefaultEditor;
use tokio::sync::Mutex;

use crate::loader::parser::IQOSConsole;

pub fn register_command(console: &mut IQOSConsole) {
    console.register_command(
        "findmyiqos",
        Box::new(|iqos, _| Box::pin(async move { execute(iqos).await })),
    );
}

async fn execute(iqos: Arc<Mutex<Iqos<IqosBle>>>) -> Result<()> {
    println!("Starting Find My IQOS...");

    {
        let iqos = iqos.lock().await;
        iqos.find_my_iqos_start().await?;
    }

    let prompt_result = tokio::task::block_in_place(|| -> Result<()> {
        let mut rl = DefaultEditor::new()?;
        let _ = rl.readline("Press <Enter> to stop"); // any input or EOF proceeds to stop
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
}
