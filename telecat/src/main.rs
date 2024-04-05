mod tcp_abridged;
mod transport;
mod session;
mod authrpc;
mod instance;
mod rpc;
mod types;

use std::error::Error;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::instance::{Instance, instance_start};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    cattl::mtproto::init();
    let instance = Arc::new(Mutex::new(Instance::new("127.0.0.1:8443".into())));
    instance_start(instance).await?;

    Ok(())
}
