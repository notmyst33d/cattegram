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

pub const CURRENT_PRIME: [u8; 256] = [
    0xc7, 0x1c, 0xae, 0xb9, 0xc6, 0xb1, 0xc9, 0x04, 0x8e, 0x6c, 0x52, 0x2f, 0x70, 0xf1, 0x3f, 0x73,
    0x98, 0x0d, 0x40, 0x23, 0x8e, 0x3e, 0x21, 0xc1, 0x49, 0x34, 0xd0, 0x37, 0x56, 0x3d, 0x93, 0x0f,
    0x48, 0x19, 0x8a, 0x0a, 0xa7, 0xc1, 0x40, 0x58, 0x22, 0x94, 0x93, 0xd2, 0x25, 0x30, 0xf4, 0xdb,
    0xfa, 0x33, 0x6f, 0x6e, 0x0a, 0xc9, 0x25, 0x13, 0x95, 0x43, 0xae, 0xd4, 0x4c, 0xce, 0x7c, 0x37,
    0x20, 0xfd, 0x51, 0xf6, 0x94, 0x58, 0x70, 0x5a, 0xc6, 0x8c, 0xd4, 0xfe, 0x6b, 0x6b, 0x13, 0xab,
    0xdc, 0x97, 0x46, 0x51, 0x29, 0x69, 0x32, 0x84, 0x54, 0xf1, 0x8f, 0xaf, 0x8c, 0x59, 0x5f, 0x64,
    0x24, 0x77, 0xfe, 0x96, 0xbb, 0x2a, 0x94, 0x1d, 0x5b, 0xcd, 0x1d, 0x4a, 0xc8, 0xcc, 0x49, 0x88,
    0x07, 0x08, 0xfa, 0x9b, 0x37, 0x8e, 0x3c, 0x4f, 0x3a, 0x90, 0x60, 0xbe, 0xe6, 0x7c, 0xf9, 0xa4,
    0xa4, 0xa6, 0x95, 0x81, 0x10, 0x51, 0x90, 0x7e, 0x16, 0x27, 0x53, 0xb5, 0x6b, 0x0f, 0x6b, 0x41,
    0x0d, 0xba, 0x74, 0xd8, 0xa8, 0x4b, 0x2a, 0x14, 0xb3, 0x14, 0x4e, 0x0e, 0xf1, 0x28, 0x47, 0x54,
    0xfd, 0x17, 0xed, 0x95, 0x0d, 0x59, 0x65, 0xb4, 0xb9, 0xdd, 0x46, 0x58, 0x2d, 0xb1, 0x17, 0x8d,
    0x16, 0x9c, 0x6b, 0xc4, 0x65, 0xb0, 0xd6, 0xff, 0x9c, 0xa3, 0x92, 0x8f, 0xef, 0x5b, 0x9a, 0xe4,
    0xe4, 0x18, 0xfc, 0x15, 0xe8, 0x3e, 0xbe, 0xa0, 0xf8, 0x7f, 0xa9, 0xff, 0x5e, 0xed, 0x70, 0x05,
    0x0d, 0xed, 0x28, 0x49, 0xf4, 0x7b, 0xf9, 0x59, 0xd9, 0x56, 0x85, 0x0c, 0xe9, 0x29, 0x85, 0x1f,
    0x0d, 0x81, 0x15, 0xf6, 0x35, 0xb1, 0x05, 0xee, 0x2e, 0x4e, 0x15, 0xd0, 0x4b, 0x24, 0x54, 0xbf,
    0x6f, 0x4f, 0xad, 0xf0, 0x34, 0xb1, 0x04, 0x03, 0x11, 0x9c, 0xd8, 0xe3, 0xb9, 0x2f, 0xcc, 0x5b,
];

pub async fn rpc_req_pq_multi(
    session: Arc<Mutex<Session>>,
    message: rpc::Message<ReqPqMulti>,
) -> Result<SchemaObject, Box<dyn Error + Send + Sync>> {
    let mut session = session.lock().await;

    session.auth_key_flow.nonce = message.obj.nonce;

    // TODO: Get random server_nonce
    session.auth_key_flow.server_nonce = -168111077373544806074672857131836382372;

    // TODO: Remove hardcoded p and q
    session.auth_key_flow.p = 1305305213;
    session.auth_key_flow.q = 1774703071;

    ok_raw!(ResPq {
        nonce: session.auth_key_flow.nonce,
        server_nonce: session.auth_key_flow.server_nonce,
        pq: (session.auth_key_flow.p as u64 * session.auth_key_flow.q as u64)
            .to_be_bytes()
            .to_vec(),
        server_public_key_fingerprints: vec![session.runtime_config.rsa_fingerprint],
    })
}

pub async fn rpc_req_dh_params(
    session: Arc<Mutex<Session>>,
    message: rpc::Message<ReqDhParams>,
) -> Result<SchemaObject, Box<dyn Error + Send + Sync>> {
    let mut session = session.lock().await;

    if message.obj.nonce != session.auth_key_flow.nonce
        || message.obj.server_nonce != session.auth_key_flow.server_nonce
    {
        return Err("nonce values altered".into());
    }

    if message.obj.public_key_fingerprint != session.runtime_config.rsa_fingerprint {
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

    let decrypted = BigUint::from_bytes_be(&message.obj.encrypted_data)
        .modpow(
            &session.runtime_config.rsa_private_exponent,
            &session.runtime_config.rsa_modulus,
        )
        .to_bytes_be();

    let mut extended_encryption = true;
    match read_p_q_inner_data_variant(&mut decrypted[20..].into()) {
        Ok(PQInnerDataVariant::PQInnerDataDc(inner_data)) => {
            session.auth_key_flow.nonce = inner_data.nonce;
            session.auth_key_flow.server_nonce = inner_data.server_nonce;
            session.auth_key_flow.new_nonce = clone_sized_slice!(&inner_data.new_nonce, 32);
            extended_encryption = false;
        }
        Ok(PQInnerDataVariant::PQInnerDataTempDc(inner_data)) => {
            session.auth_key_flow.nonce = inner_data.nonce;
            session.auth_key_flow.server_nonce = inner_data.server_nonce;
            session.auth_key_flow.new_nonce = clone_sized_slice!(&inner_data.new_nonce, 32);
            extended_encryption = false;
        }
        Ok(PQInnerDataVariant::PQInnerData(inner_data)) => {
            session.auth_key_flow.nonce = inner_data.nonce;
            session.auth_key_flow.server_nonce = inner_data.server_nonce;
            session.auth_key_flow.new_nonce = clone_sized_slice!(&inner_data.new_nonce, 32);
            extended_encryption = false;
        }
        Ok(PQInnerDataVariant::PQInnerDataTemp(inner_data)) => {
            session.auth_key_flow.nonce = inner_data.nonce;
            session.auth_key_flow.server_nonce = inner_data.server_nonce;
            session.auth_key_flow.new_nonce = clone_sized_slice!(&inner_data.new_nonce, 32);
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
                session.auth_key_flow.nonce = inner_data.nonce;
                session.auth_key_flow.server_nonce = inner_data.server_nonce;
                session.auth_key_flow.new_nonce = clone_sized_slice!(&inner_data.new_nonce, 32);
            }
            PQInnerDataVariant::PQInnerDataTempDc(inner_data) => {
                session.auth_key_flow.nonce = inner_data.nonce;
                session.auth_key_flow.server_nonce = inner_data.server_nonce;
                session.auth_key_flow.new_nonce = clone_sized_slice!(&inner_data.new_nonce, 32);
            }
            PQInnerDataVariant::PQInnerData(inner_data) => {
                session.auth_key_flow.nonce = inner_data.nonce;
                session.auth_key_flow.server_nonce = inner_data.server_nonce;
                session.auth_key_flow.new_nonce = clone_sized_slice!(&inner_data.new_nonce, 32);
            }
            PQInnerDataVariant::PQInnerDataTemp(inner_data) => {
                session.auth_key_flow.nonce = inner_data.nonce;
                session.auth_key_flow.server_nonce = inner_data.server_nonce;
                session.auth_key_flow.new_nonce = clone_sized_slice!(&inner_data.new_nonce, 32);
            }
        }
    }

    session.auth_key_flow.tmp_aes_key = {
        let mut out = [0u8; 32];
        let n1 = sha1!(
            &session.auth_key_flow.new_nonce,
            &session.auth_key_flow.server_nonce.to_le_bytes()
        );
        let n2 = sha1!(
            &session.auth_key_flow.server_nonce.to_le_bytes(),
            &session.auth_key_flow.new_nonce
        );
        out[..20].clone_from_slice(&n1);
        out[20..].clone_from_slice(&n2[..12]);
        out
    };

    session.auth_key_flow.tmp_aes_iv = {
        let mut out = [0u8; 32];
        let n1 = sha1!(
            &session.auth_key_flow.server_nonce.to_le_bytes(),
            &session.auth_key_flow.new_nonce
        );
        let n2 = sha1!(
            &session.auth_key_flow.new_nonce,
            &session.auth_key_flow.new_nonce
        );
        out[..8].clone_from_slice(&n1[12..]);
        out[8..28].clone_from_slice(&n2);
        out[28..].clone_from_slice(&session.auth_key_flow.new_nonce[..4]);
        out
    };

    // TODO: Add random
    session.auth_key_flow.a = BigUint::from_bytes_le(&[123u8; 256]);
    session.auth_key_flow.g_a = session.auth_key_flow.g.to_biguint().unwrap().modpow(
        &session.auth_key_flow.a,
        &BigUint::from_bytes_be(&CURRENT_PRIME),
    );

    let mut inner_data = TlBuffer::new(vec![]);
    ServerDhInnerData {
        nonce: session.auth_key_flow.nonce,
        server_nonce: session.auth_key_flow.server_nonce,
        g: session.auth_key_flow.g,
        dh_prime: CURRENT_PRIME.to_vec(),
        g_a: session.auth_key_flow.g_a.to_bytes_be().to_vec(),
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
        ige_encrypt(
            &mut out,
            &session.auth_key_flow.tmp_aes_key,
            &session.auth_key_flow.tmp_aes_iv,
        );
        out
    };

    ok_raw!(ServerDhParamsOk {
        nonce: session.auth_key_flow.nonce,
        server_nonce: session.auth_key_flow.server_nonce,
        encrypted_answer,
    })
}

pub async fn rpc_set_client_dh_params(
    session: Arc<Mutex<Session>>,
    message: rpc::Message<SetClientDhParams>,
) -> Result<SchemaObject, Box<dyn Error + Send + Sync>> {
    let session = session.lock().await;

    let buffer = &ige_decrypt(
        &message.obj.encrypted_data,
        &session.auth_key_flow.tmp_aes_key,
        &session.auth_key_flow.tmp_aes_iv,
    )[24..];
    let inner_data = read_client_dh_inner_data(&mut buffer.into())?;

    let auth_key = clone_sized_slice!(
        &BigUint::from_bytes_be(&inner_data.g_b)
            .modpow(
                &session.auth_key_flow.a,
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
        &sha1!(&session.auth_key_flow.new_nonce, [0x01], &auth_key_aux_hash)[4..],
        16
    ));

    session
        .storage
        .insert_auth_key(auth_key_id, auth_key)
        .await?;

    ok_raw!(DhGenOk {
        nonce: session.auth_key_flow.nonce,
        server_nonce: session.auth_key_flow.server_nonce,
        new_nonce_hash1,
    })
}
