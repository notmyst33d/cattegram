use std::error::Error;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use cattl::TlObject;
use cattl::mtproto::*;
use crate::tcp_abridged::TcpAbridged;
use crate::session::Session;
use crate::rpc::{RpcResult, RpcMapping};
use crate::authrpc;

pub struct Instance {
    address: String,
    rpc: RpcMapping,
    authrpc: RpcMapping,
}

impl Instance {
    pub fn new(address: String) -> Self {
        let mut s = Self {
            address,
            rpc: vec![],
            authrpc: vec![],
        };
        s.authrpc.extend(authrpc::mapping());
        s
    }
}

pub async fn instance_invoke_rpc(instance: Arc<Mutex<Instance>>, session: Arc<Mutex<Session>>, obj: &(dyn TlObject + Sync)) -> RpcResult {
    let hash = obj.hash();

    for rpc in &instance.lock().await.rpc {
        if rpc.0 == hash {
            return rpc.1(session, obj).await;
        }
    }

    Err(Box::new("No such RPC method"))
}

pub async fn instance_start(instance: Arc<Mutex<Instance>>) -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(&instance.lock().await.address).await?;
    println!("TCP Server started");

    loop {
        let (socket, _) = listener.accept().await?;
        let instance_clone = instance.clone();
        tokio::spawn(async move {
            match instance_client_thread(instance_clone, socket).await {
                Ok(_) => {},
                Err(_) => {
                    println!("Client returned an error");
                },
            }
        });
    }
}

async fn instance_client_thread(instance: Arc<Mutex<Instance>>, mut socket: TcpStream) -> Result<(), Box<dyn Error>> {
    let mut buf = [0u8; 1];

    socket.read_exact(&mut buf[..1]).await?;

    if buf[0] != 0xef {
        println!("Unsupported transport, only TCP Abridged is supported currently");
        socket.shutdown().await?;
        return Ok(());
    }

    let mut session = Arc::new(Mutex::new(Session::new(Box::new(TcpAbridged::new(socket)))));
    let obj = session.lock().await.receive_raw::<req_pq_multi>().await?;
    println!("nonce: {}", obj.nonce);

    instance_invoke_rpc(instance.clone(), session.clone(), &obj).await;

    session.lock().await.close().await?;

    Ok(())
}
