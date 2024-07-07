use crate::prime::CURRENT_PRIME;
use crate::rsa::{rsa_decrypt, FINGERPRINT};
use crate::session::Session;
use crate::{clone_sized_slice, ok_raw, rpc, time};
use catte_tl_buffer::TlBuffer;
use catte_tl_schema::*;
use grammers_crypto::aes::{ige_decrypt, ige_encrypt};
use grammers_crypto::{sha1, sha256};
use num_bigint::{BigUint, ToBigUint};
use std::error::Error;
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn rpc_req_pq_multi(
    session: Arc<Mutex<Session>>,
    message: rpc::Message<ReqPqMulti>,
) -> Result<SchemaObject, Box<dyn Error + Send + Sync>> {
    let flow = &mut session.lock().await.auth_key_flow;
    flow.nonce = message.obj.nonce;

    // TODO: Get random server_nonce
    flow.server_nonce = -168111077373544806074672857131836382372;

    // TODO: Remove hardcoded p and q
    flow.p = 1305305213;
    flow.q = 1774703071;

    ok_raw!(ResPq {
        nonce: flow.nonce,
        server_nonce: flow.server_nonce,
        pq: (flow.p as u64 * flow.q as u64).to_be_bytes().to_vec(),
        server_public_key_fingerprints: vec![FINGERPRINT as i64],
    })
}

pub async fn rpc_req_dh_params(
    session: Arc<Mutex<Session>>,
    message: rpc::Message<ReqDhParams>,
) -> Result<SchemaObject, Box<dyn Error + Send + Sync>> {
    let flow = &mut session.lock().await.auth_key_flow;
    if message.obj.nonce != flow.nonce || message.obj.server_nonce != flow.server_nonce {
        return Err("nonce values altered".into());
    }

    if message.obj.public_key_fingerprint != FINGERPRINT as i64 {
        return Err("unknown fingerprint".into());
    }

    let mut convert_buffer = [0u8; 4];
    convert_buffer.clone_from_slice(&message.obj.p);
    let cp = u32::from_be_bytes(convert_buffer) as u64;
    convert_buffer.clone_from_slice(&message.obj.q);
    let cq = u32::from_be_bytes(convert_buffer) as u64;
    if cp > cq {
        return Err("pq factorization failed".into());
    }

    let mut extended_encryption = true;
    let decrypted = rsa_decrypt(&message.obj.encrypted_data);
    match read_p_q_inner_data_variant(&mut decrypted[20..].into()) {
        Ok(PQInnerDataVariant::PQInnerDataDc(inner_data)) => {
            flow.nonce = inner_data.nonce;
            flow.server_nonce = inner_data.server_nonce;
            flow.new_nonce = clone_sized_slice!(&inner_data.new_nonce, 32);
            extended_encryption = false;
        }
        Ok(PQInnerDataVariant::PQInnerDataTempDc(inner_data)) => {
            flow.nonce = inner_data.nonce;
            flow.server_nonce = inner_data.server_nonce;
            flow.new_nonce = clone_sized_slice!(&inner_data.new_nonce, 32);
            extended_encryption = false;
        }
        Ok(PQInnerDataVariant::PQInnerData(inner_data)) => {
            flow.nonce = inner_data.nonce;
            flow.server_nonce = inner_data.server_nonce;
            flow.new_nonce = clone_sized_slice!(&inner_data.new_nonce, 32);
            extended_encryption = false;
        }
        Ok(PQInnerDataVariant::PQInnerDataTemp(inner_data)) => {
            flow.nonce = inner_data.nonce;
            flow.server_nonce = inner_data.server_nonce;
            flow.new_nonce = clone_sized_slice!(&inner_data.new_nonce, 32);
            extended_encryption = false;
        }
        Err(..) => {}
    }

    // I hate this with great passion, but this is required
    // in order for official clients to work correctly
    // https://core.telegram.org/mtproto/auth_key#41-rsa-paddata-server-public-key-mentioned-above-is-implemented-as-follows
    if extended_encryption {
        let temp_key_xor = &decrypted[..32];
        let aes_encrypted = &decrypted[32..];
        let sha256_aes_encrypted = sha256!(aes_encrypted);
        let temp_key = temp_key_xor
            .iter()
            .zip(sha256_aes_encrypted.iter())
            .map(|(&v1, &v2)| v1 ^ v2)
            .collect::<Vec<_>>();
        let data_with_hash = ige_decrypt(
            aes_encrypted,
            &clone_sized_slice!(&temp_key, 32),
            &[0u8; 32],
        );
        let data = data_with_hash[..data_with_hash.len() - 32]
            .iter()
            .cloned()
            .rev()
            .collect::<Vec<_>>();
        match read_p_q_inner_data_variant(&mut data.into())? {
            PQInnerDataVariant::PQInnerDataDc(inner_data) => {
                flow.nonce = inner_data.nonce;
                flow.server_nonce = inner_data.server_nonce;
                flow.new_nonce = clone_sized_slice!(&inner_data.new_nonce, 32);
            }
            PQInnerDataVariant::PQInnerDataTempDc(inner_data) => {
                flow.nonce = inner_data.nonce;
                flow.server_nonce = inner_data.server_nonce;
                flow.new_nonce = clone_sized_slice!(&inner_data.new_nonce, 32);
            }
            PQInnerDataVariant::PQInnerData(inner_data) => {
                flow.nonce = inner_data.nonce;
                flow.server_nonce = inner_data.server_nonce;
                flow.new_nonce = clone_sized_slice!(&inner_data.new_nonce, 32);
            }
            PQInnerDataVariant::PQInnerDataTemp(inner_data) => {
                flow.nonce = inner_data.nonce;
                flow.server_nonce = inner_data.server_nonce;
                flow.new_nonce = clone_sized_slice!(&inner_data.new_nonce, 32);
            }
        }
    }

    flow.tmp_aes_key = {
        let mut out = [0u8; 32];
        let n1 = sha1!(&flow.new_nonce, &flow.server_nonce.to_le_bytes());
        let n2 = sha1!(&flow.server_nonce.to_le_bytes(), &flow.new_nonce);
        out[..20].clone_from_slice(&n1);
        out[20..].clone_from_slice(&n2[..12]);
        out
    };

    flow.tmp_aes_iv = {
        let mut out = [0u8; 32];
        let n1 = sha1!(&flow.server_nonce.to_le_bytes(), &flow.new_nonce);
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

    let mut inner_data = TlBuffer::new(vec![]);
    ServerDhInnerData {
        nonce: flow.nonce,
        server_nonce: flow.server_nonce,
        g: flow.g,
        dh_prime: CURRENT_PRIME.to_vec(),
        g_a: flow.g_a.to_bytes_be().to_vec(),
        server_time: time!(),
    }
    .write(&mut inner_data);

    let encrypted_answer = {
        let padding_data = [0u8; 16];
        let mut out = vec![];
        out.extend(sha1!(&inner_data.data()));
        out.extend(inner_data.data());
        let padding = 16 - (out.len() % 16);
        out.extend(&padding_data[..padding]);
        ige_encrypt(&mut out, &flow.tmp_aes_key, &flow.tmp_aes_iv);
        out
    };

    ok_raw!(ServerDhParamsOk {
        nonce: flow.nonce,
        server_nonce: flow.server_nonce,
        encrypted_answer,
    })
}

pub async fn rpc_set_client_dh_params(
    session: Arc<Mutex<Session>>,
    message: rpc::Message<SetClientDhParams>,
) -> Result<SchemaObject, Box<dyn Error + Send + Sync>> {
    let locked_session = &mut session.lock().await;

    let buffer = &ige_decrypt(
        &message.obj.encrypted_data,
        &locked_session.auth_key_flow.tmp_aes_key,
        &locked_session.auth_key_flow.tmp_aes_iv,
    )[24..];
    let inner_data = read_client_dh_inner_data(&mut buffer.into())?;

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
    let auth_key_id = i64::from_le_bytes(clone_sized_slice!(
        &auth_key_sha[auth_key_sha.len() - 8..],
        8
    ));
    let auth_key_aux_hash = &auth_key_sha[..8];

    let new_nonce_hash1 = i128::from_le_bytes(clone_sized_slice!(
        &sha1!(
            &locked_session.auth_key_flow.new_nonce,
            [0x01],
            &auth_key_aux_hash
        )[4..],
        16
    ));

    locked_session
        .storage
        .store_auth_key(auth_key_id, auth_key)?;

    ok_raw!(DhGenOk {
        nonce: locked_session.auth_key_flow.nonce,
        server_nonce: locked_session.auth_key_flow.server_nonce,
        new_nonce_hash1,
    })
}
