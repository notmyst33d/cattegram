use crate::impl_rpc;
use crate::prime::CURRENT_PRIME;
use crate::rpc::{RpcMapping, RpcResult};
use crate::rsa::{rsa_decrypt, FINGERPRINT};
use crate::session::Session;
use crate::unsafe_cast;
use cattl::{mtproto::*, BytesBuffer, TlObject};
use grammers_crypto::aes::{ige_decrypt, ige_encrypt};
use grammers_crypto::sha1;
use num_bigint::{BigUint, ToBigUint};
use std::any::Any;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;

macro_rules! clone_sized_slice {
    ($v:expr, $s:expr) => {{
        let mut s = [0u8; $s];
        s.clone_from_slice($v);
        s
    }};
}

macro_rules! hex_string {
    ($v:expr) => {
        $v.iter().map(|v| format!("{:02x}", v)).collect::<String>()
    };
}

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
        server_public_key_fingerprints: vec![FINGERPRINT as i64],
    }))
}

pub async fn rpc_req_dh_params(session: Arc<Mutex<Session>>, req: Box<req_DH_params>) -> RpcResult {
    let flow = &mut session.lock().await.auth_key_flow;
    if req.nonce != flow.nonce || req.server_nonce != flow.server_nonce {
        return Err("nonce values altered".into());
    }

    if req.public_key_fingerprint != FINGERPRINT as i64 {
        return Err("unknown fingerprint".into());
    }

    let mut convert_buffer = [0u8; 4];
    convert_buffer.clone_from_slice(&req.p);
    let cp = u32::from_be_bytes(convert_buffer) as u64;
    convert_buffer.clone_from_slice(&req.q);
    let cq = u32::from_be_bytes(convert_buffer) as u64;
    if cp > cq {
        return Err("pq factorization failed".into());
    }

    let mut decrypted = BytesBuffer::new(rsa_decrypt(&req.encrypted_data));
    decrypted.seek(24);

    let inner_data = read_p_q_inner_data_dc(&mut decrypted)?;
    flow.nonce = inner_data.nonce;
    flow.server_nonce = inner_data.server_nonce;
    flow.new_nonce = clone_sized_slice!(&inner_data.new_nonce, 32);

    let server_nonce_bytes = flow.server_nonce.to_le_bytes();
    flow.tmp_aes_key = {
        let mut out = [0u8; 32];
        let n1 = sha1!(&flow.new_nonce, &server_nonce_bytes);
        let n2 = sha1!(&server_nonce_bytes, &flow.new_nonce);
        out[..20].clone_from_slice(&n1);
        out[20..].clone_from_slice(&n2[..12]);
        out
    };

    flow.tmp_aes_iv = {
        let mut out = [0u8; 32];
        let n1 = sha1!(&server_nonce_bytes, &flow.new_nonce);
        let n2 = sha1!(&flow.new_nonce, &flow.new_nonce);
        out[..8].clone_from_slice(&n1[12..]);
        out[8..28].clone_from_slice(&n2);
        out[28..].clone_from_slice(&flow.new_nonce[..4]);
        out
    };

    // TODO: Add random
    flow.a = BigUint::from_bytes_le(&[123u8; 256]);
    flow.g_a = flow
        .g
        .to_biguint()
        .unwrap()
        .modpow(&flow.a, &BigUint::from_bytes_be(&CURRENT_PRIME));

    let mut data_buffer = BytesBuffer::new(vec![]);
    server_DH_inner_data {
        nonce: flow.nonce,
        server_nonce: flow.server_nonce,
        g: flow.g,
        dh_prime: CURRENT_PRIME.to_vec(),
        g_a: flow.g_a.to_bytes_be().to_vec(),
        server_time: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i32,
    }
    .write(&mut data_buffer);

    let encrypted_answer = {
        let data = data_buffer.data();
        let padding_data = [0u8; 16];
        let mut out = vec![];
        out.extend(sha1!(data));
        out.extend(data);
        let padding = 16 - (out.len() % 16);
        out.extend(&padding_data[..padding]);
        ige_encrypt(&mut out, &flow.tmp_aes_key, &flow.tmp_aes_iv);
        out
    };

    Ok(Box::new(server_DH_params_ok {
        nonce: flow.nonce,
        server_nonce: flow.server_nonce,
        encrypted_answer,
    }))
}

pub async fn rpc_set_client_dh_params(
    session: Arc<Mutex<Session>>,
    req: Box<set_client_DH_params>,
) -> RpcResult {
    let locked_session = &mut session.lock().await;

    let mut buffer = BytesBuffer::new(ige_decrypt(
        &req.encrypted_data,
        &locked_session.auth_key_flow.tmp_aes_key,
        &locked_session.auth_key_flow.tmp_aes_iv,
    ));
    buffer.seek(24);
    let inner_data = read_client_DH_inner_data(&mut buffer)?;

    let auth_key = clone_sized_slice!(
        &BigUint::from_bytes_be(&inner_data.g_b)
            .modpow(
                &locked_session.auth_key_flow.a,
                &BigUint::from_bytes_be(&CURRENT_PRIME)
            )
            .to_bytes_be(),
        256
    );
    let auth_key_sha = sha1!(auth_key);
    let auth_key_id = clone_sized_slice!(&auth_key_sha[auth_key_sha.len() - 8..], 8);
    let auth_key_aux_hash = &auth_key_sha[..8];

    let new_nonce_hash1 = i128::from_le_bytes(clone_sized_slice!(
        &sha1!(
            &locked_session.auth_key_flow.new_nonce,
            [0x01],
            &auth_key_aux_hash
        )[4..],
        16
    ));

    println!("Finished auth_key creation");
    println!("auth_key_id: {}", hex_string!(auth_key_id));
    println!("auth_key: {}", hex_string!(auth_key));

    locked_session.auth_key = auth_key;
    locked_session.auth_key_id = auth_key_id;

    Ok(Box::new(dh_gen_ok {
        nonce: locked_session.auth_key_flow.nonce,
        server_nonce: locked_session.auth_key_flow.server_nonce,
        new_nonce_hash1,
    }))
}

pub fn mapping() -> RpcMapping {
    vec![
        impl_rpc!(req_pq_multi::hash(), rpc_req_pq_multi, req_pq_multi),
        impl_rpc!(req_DH_params::hash(), rpc_req_dh_params, req_DH_params),
        impl_rpc!(
            set_client_DH_params::hash(),
            rpc_set_client_dh_params,
            set_client_DH_params
        ),
    ]
}
