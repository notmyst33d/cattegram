use std::sync::Arc;
use std::any::Any;
use tokio::sync::Mutex;
use cattl::mtproto::*;
use crate::session::Session;
use crate::rpc::{RpcResult, RpcMapping};

pub async fn rpc_req_pq_multi<'a>(session: Arc<Mutex<Session>>, req: Box<req_pq_multi>) -> RpcResult {
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
        (-1099002127, |session, obj| Box::pin(rpc_req_pq_multi(session, (obj as Box<dyn Any>).downcast::<req_pq_multi>().unwrap()))),
    ]
}
