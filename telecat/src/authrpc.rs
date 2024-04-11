use std::sync::Arc;
use std::any::Any;
use tokio::sync::Mutex;
use cattl::mtproto::*;
use crate::session::Session;
use crate::rpc::{RpcResult, RpcMapping};
use crate::impl_rpc;

pub async fn rpc_req_pq_multi(session: Arc<Mutex<Session>>, req: Box<req_pq_multi>) -> RpcResult {
    println!("req_pq_multi nonce={}", req.nonce);
    Ok(Box::new(resPQ {
        nonce: req.nonce,
        server_nonce: 0,
        pq: b"AAAAAAAAA",
        server_public_key_fingerprints: vec![123, 321],
    }))
}

pub fn mapping() -> RpcMapping {
    vec![
        impl_rpc!(-1099002127, rpc_req_pq_multi, req_pq_multi),
    ]
}
