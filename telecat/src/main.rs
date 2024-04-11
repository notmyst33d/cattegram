#![feature(trait_upcasting)]
#![feature(allocator_api)]

mod tcp_abridged;
mod transport;
mod session;
mod authrpc;
mod instance;
mod rpc;

use std::error::Error;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use crate::instance::Instance;
use crate::tcp_abridged::TcpAbridged;
use crate::session::Session;

async fn client_thread(instance: Arc<Instance>, mut socket: TcpStream) -> Result<(), Box<dyn Error>> {
    let mut buf = [0u8; 1];

    socket.read_exact(&mut buf[..1]).await?;

    if buf[0] != 0xef {
        println!("Unsupported transport, only TCP Abridged is supported currently");
        socket.shutdown().await?;
        return Ok(());
    }

    let session = Arc::new(Mutex::new(Session::new(Box::new(TcpAbridged::new(socket)))));

    loop {
        let req = session.lock().await.receive_raw().await?;
        let res = instance.invoke_authrpc(session.clone(), req).await?;
        session.lock().await.send_raw(res).await?;
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let instance = Arc::new(Instance::new("127.0.0.1:8443".into()));
    let listener = TcpListener::bind(&instance.address).await?;
    println!("Instance started on {}", instance.address);

    loop {
        let (socket, _) = listener.accept().await?;
        let instance_clone = instance.clone();
        tokio::spawn(async move {
            match client_thread(instance_clone, socket).await {
                Ok(_) => {},
                Err(e) => println!("Client returned an error: {}", e),
            }
        });
    }
}
