use std::future::Future;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;

use anyhow::Result;
use iqos::{Iqos, IqosBle};
use tokio::sync::Mutex;

pub type CommandFn = Box<
    dyn Fn(Arc<Mutex<Iqos<IqosBle>>>, Vec<String>) -> Pin<Box<dyn Future<Output = Result<()>> + Send>>
        + Send
        + Sync,
>;

pub type CommandRegistry = HashMap<String, CommandFn>;

