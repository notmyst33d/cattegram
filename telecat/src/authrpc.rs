use std::sync::Arc;
use std::any::Any;
use tokio::sync::Mutex;
use cattl::mtproto::*;
use crate::session::Session;
use crate::rpc::{RpcResult, RpcMapping};
use crate::impl_rpc;

pub async fn rpc_req_pq_multi(session: Arc<Mutex<Session>>, req: Box<req_pq_multi>) -> RpcResult {
    let flow = &mut session.lock().await.auth_key_flow;
    flow.nonce = req.nonce;

    // TODO: Get random server_nonce
    flow.server_nonce = 0;

    // TODO: Remove temporary hardcoded p and q
    flow.p = 1305305213;
    flow.q = 1774703071;

    Ok(Box::new(resPQ {
        nonce: flow.nonce,
        server_nonce: flow.server_nonce,
        pq: (flow.p as u64 * flow.q as u64).to_be_bytes().to_vec(),
        server_public_key_fingerprints: vec![-4344800451088585951], // TODO: Remove hardcoded fingerprint
    }))
}

pub async fn rpc_req_DH_params(session: Arc<Mutex<Session>>, req: Box<req_DH_params>) -> RpcResult {
    let flow = &mut session.lock().await.auth_key_flow;
    if req.nonce != flow.nonce || req.server_nonce != flow.server_nonce {
        return Err("nonce values altered".into())
    }

    if req.public_key_fingerprint != -4344800451088585951 { // TODO: Remove hardcoded fingerprint
        return Err("unknown fingerprint".into())
    }

    let mut convert_buffer = [0u8; 4];
    convert_buffer.clone_from_slice(&req.p);
    let cp = u32::from_be_bytes(convert_buffer) as u64;
    convert_buffer.clone_from_slice(&req.q);
    let cq = u32::from_be_bytes(convert_buffer) as u64;
    if cp > cq {
        return Err("pq factorization failed".into())
    }

    Ok(Box::new(server_DH_params_ok {
        nonce: flow.nonce,
        server_nonce: flow.server_nonce,
        encrypted_answer: b"No".to_vec(),
    }))
}

pub fn mapping() -> RpcMapping {
    vec![
        impl_rpc!(req_pq_multi::hash(), rpc_req_pq_multi, req_pq_multi),
        impl_rpc!(req_DH_params::hash(), rpc_req_DH_params, req_DH_params),
    ]
}
