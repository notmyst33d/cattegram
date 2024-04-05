use std::sync::Arc;
use tokio::sync::Mutex;
use cattl::TlObject;
use cattl::mtproto::*;
use crate::session::Session;
use crate::rpc::{RpcResult, RpcMapping};

pub async fn rpc_req_pq_multi(session: Arc<Mutex<Session>>, obj: &(dyn TlObject + Sync)) -> RpcResult {
    Ok(Box::new(resPQ {
        nonce: 0,
        server_nonce: 0,
        pq: b"AAAAAAAAA",
        server_public_key_fingerprints: vec![123, 321],
    }))
}

pub fn mapping() -> RpcMapping {
    vec![
        (-1099002127, |session, obj| Box::pin(rpc_req_pq_multi(session, obj))),
    ]
}
