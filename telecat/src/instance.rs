use std::error::Error;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use cattl::TlObject;
use crate::tcp_abridged::TcpAbridged;
use crate::session::Session;
use crate::rpc::{RpcResult, RpcMapping};
use crate::authrpc;

pub struct Instance {
    pub address: String,
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

    pub async fn invoke_rpc(&self, session: Arc<Mutex<Session>>, obj: Box<dyn TlObject + Send + Sync>) -> RpcResult {
        let hash = obj.hash();

        for rpc in &self.rpc {
            if rpc.0 == hash {
                return rpc.1(session, obj).await;
            }
        }

        Err("No such RPC method".into())
    }

    pub async fn invoke_authrpc(&self, session: Arc<Mutex<Session>>, obj: Box<dyn TlObject + Send + Sync>) -> RpcResult {
        let hash = obj.hash();

        for rpc in &self.authrpc {
            if rpc.0 == hash {
                return rpc.1(session, obj).await;
            }
        }

        Err("No such auth RPC method".into())
    }

}

