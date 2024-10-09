use crate::RuntimeConfig;
use crate::{clone_sized_slice, storage::Storage, transport::Transport, Config as ServerConfig};
use catte_tl_buffer::TlBuffer;
use catte_tl_schema::*;
use flate2::read::GzDecoder;
use grammers_crypto::{decrypt_data_server_v2, encrypt_data_server_v2, AuthKey, DequeBuffer};
use num_bigint::BigUint;
use std::sync::Arc;
use std::{
    error::Error,
    io::Read,
    time::{SystemTime, UNIX_EPOCH},
};

pub struct AuthKeyFlow {
    pub nonce: i128,
    pub server_nonce: i128,
    pub new_nonce: [u8; 32],
    pub tmp_aes_key: [u8; 32],
    pub tmp_aes_iv: [u8; 32],
    pub p: u32,
    pub q: u32,
    pub a: BigUint,
    pub g: i32,
    pub g_a: BigUint,
}

impl AuthKeyFlow {
    pub fn new() -> Self {
        Self {
            nonce: 0,
            server_nonce: 0,
            new_nonce: [0u8; 32],
            tmp_aes_key: [0u8; 32],
            tmp_aes_iv: [0u8; 32],
            p: 0,
            q: 0,
            a: BigUint::default(),
            g: 3,
            g_a: BigUint::default(),
        }
    }
}

pub struct LoginFlow {
    pub phone_number: String,
    pub phone_number_verified: bool,
    pub code: String,
}

impl LoginFlow {
    pub fn new() -> Self {
        Self {
            phone_number: String::new(),
            phone_number_verified: false,
            code: String::new(),
        }
    }
}

pub struct Session {
    pub closed: bool,
    pub storage: Storage,
    pub authorized: bool,
    pub encrypted: bool,
    pub auth_key_flow: AuthKeyFlow,
    pub auth_key_id: i64,
    pub auth_key: AuthKey,
    pub login_flow: LoginFlow,
    pub id: i64,
    pub seq_no: i32,
    pub config: Arc<ServerConfig>,
    pub runtime_config: Arc<RuntimeConfig>,
    transport: Box<dyn Transport>,
    last_msg_id: i64,
}

impl Session {
    pub async fn new(
        config: Arc<ServerConfig>,
        runtime_config: Arc<RuntimeConfig>,
        transport: Box<dyn Transport>,
    ) -> Self {
        Self {
            closed: false,
            storage: Storage::new(config.data.clone()).await,
            authorized: false,
            encrypted: false,
            auth_key_flow: AuthKeyFlow::new(),
            auth_key_id: 0,
            auth_key: AuthKey::from_bytes([0u8; 256]),
            login_flow: LoginFlow::new(),
            id: 0,
            seq_no: 0,
            config,
            runtime_config,
            transport,
            last_msg_id: 0,
        }
    }

    pub async fn receive(
        &mut self,
    ) -> Result<Vec<(i64, i32, SchemaObject)>, Box<dyn Error + Send + Sync>> {
        let (raw, quick_ack) = self.transport.read().await?;
        let auth_key_id = i64::from_le_bytes(clone_sized_slice!(&raw[..8], 8));

        if !self.encrypted && auth_key_id != 0 {
            self.storage.get_auth_key(auth_key_id).await.unwrap();
            if let Ok(auth_key) = self.storage.get_auth_key(auth_key_id).await {
                self.auth_key = AuthKey::from_bytes(auth_key);
                self.auth_key_id = auth_key_id;
                self.encrypted = true;
            } else {
                self.close().await?;
                return Err(format!("cannot find auth_key for {}", auth_key_id).into());
            }
        }

        if self.encrypted {
            let (raw_data, ack_token) = decrypt_data_server_v2(&raw, &self.auth_key)?;

            let mut data = TlBuffer::new(raw_data);
            let _salt = data.read_long()?; // TODO: Do something with the salt
            let session_id = data.read_long()?;
            let msg_id = data.read_long()?;
            let seq_no = data.read_int()?;
            let length = data.read_int()?;

            if quick_ack {
                self.transport.write_quick_ack(ack_token).await?;
            }

            if session_id == 0 {
                return Err("cannot have session_id == 0".into());
            }

            if self.id == 0 && session_id != 0 {
                self.id = session_id;
                if let Ok(_) = self.storage.get_user_by_session_id(self.auth_key_id).await {
                    self.authorized = true;
                }
            }

            if self.id != session_id {
                return Err("session_id changed".into());
            }

            fn read_data(
                msg_id: i64,
                seq_no: i32,
                data: Vec<u8>,
            ) -> Result<Vec<(i64, i32, SchemaObject)>, Box<dyn Error + Send + Sync>> {
                match catte_tl_schema::read(&mut data.into()) {
                    Ok(result) => match result {
                        SchemaObject::MsgContainer(messages) => Ok(messages
                            .into_iter()
                            .map(
                                |m| -> Result<
                                    Vec<(i64, i32, SchemaObject)>,
                                    Box<dyn Error + Send + Sync>,
                                > {
                                    Ok(match m.2 {
                                        SchemaObject::GzipPacked(obj) => {
                                            let mut decoder = GzDecoder::new(&obj.packed_data[..]);
                                            let mut unpacked = vec![];
                                            decoder.read_to_end(&mut unpacked)?;
                                            read_data(msg_id, seq_no, unpacked)?
                                        }
                                        _ => vec![m],
                                    })
                                },
                            )
                            .collect::<Result<Vec<_>, _>>()?
                            .into_iter()
                            .flatten()
                            .collect::<Vec<_>>()),
                        SchemaObject::GzipPacked(obj) => {
                            let mut decoder = GzDecoder::new(&obj.packed_data[..]);
                            let mut unpacked = vec![];
                            decoder.read_to_end(&mut unpacked)?;
                            read_data(msg_id, seq_no, unpacked)
                        }
                        obj => Ok(vec![(msg_id, seq_no, obj)]),
                    },
                    Err(e) => Ok(vec![(
                        msg_id,
                        seq_no,
                        SchemaObject::DeserializationError(e),
                    )]),
                }
            }

            read_data(msg_id, seq_no, data.read_raw(length as usize)?)
        } else {
            let mut data: TlBuffer = raw.into();
            data.seek(20);
            Ok(vec![(0, 0, catte_tl_schema::read(&mut data)?)])
        }
    }

    pub async fn send(
        &mut self,
        messages: Vec<SchemaObject>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        if self.encrypted {
            let mut data = TlBuffer::new(vec![]);
            data.write_long(0);
            data.write_long(self.id);
            data.write_long(self.get_msg_id());
            data.write_int(self.get_seq_no());
            if messages.len() > 1 {
                let mut obj_buf = TlBuffer::new(vec![]);
                obj_buf.write_int(1945237724);
                obj_buf.write_int(messages.len() as i32);
                for message in messages {
                    message.write(&mut obj_buf);
                }
                data.write_int(obj_buf.len() as i32 + 8);
                data.write_raw(&obj_buf.data());
            } else {
                let mut obj_buf = TlBuffer::new(vec![]);
                messages[0].write(&mut obj_buf);
                data.write_int(obj_buf.len() as i32 + 8);
                data.write_raw(&obj_buf.data());
            }

            let mut ring_buffer = DequeBuffer::with_capacity(data.data().len(), 0);
            ring_buffer.extend(data.data());
            encrypt_data_server_v2(&mut ring_buffer, &self.auth_key);
            self.transport.write(ring_buffer.as_ref()).await?;
        } else {
            let mut data = TlBuffer::new(vec![]);
            data.write_long(0);
            data.write_long(self.get_msg_id());
            let mut obj_buf = TlBuffer::new(vec![]);
            messages[0].write(&mut obj_buf);
            data.write_int(obj_buf.len() as i32);
            data.write_raw(obj_buf.data());
            self.transport.write(&data.data()).await?;
        }
        Ok(())
    }

    fn get_msg_id(&mut self) -> i64 {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

        let seconds = (now.as_secs() as i32) as u64;
        let nanoseconds = now.subsec_nanos() as u64;
        let mut msg_id = ((seconds << 32) | (nanoseconds << 2)) as i64;

        if self.last_msg_id >= msg_id {
            msg_id = self.last_msg_id + 4;
        }

        self.last_msg_id = msg_id;
        msg_id + 1
    }

    fn get_seq_no(&mut self) -> i32 {
        self.seq_no += 1;
        self.seq_no - 1
    }

    pub async fn get_self(&self) -> Result<User, sqlx::Error> {
        let mut u = self.storage.get_user_by_session_id(self.auth_key_id).await?;
        u.is_self = true;
        Ok(u)
    }

    pub async fn get_self_peer(&self) -> Result<InputPeerUser, sqlx::Error> {
        Ok(InputPeerUser {
            user_id: self.get_self().await?.id,
            access_hash: 0,
        })
    }

    pub async fn get_self_full(&self) -> Result<(User, UserFull), sqlx::Error> {
        let user = self.get_self().await?;
        Ok((
            user.clone(),
            UserFull {
                blocked: false,
                phone_calls_available: false,
                phone_calls_private: false,
                can_pin_message: true,
                has_scheduled: false,
                video_calls_available: false,
                voice_messages_forbidden: false,
                translations_disabled: false,
                id: user.id,
                about: None,
                settings: PeerSettings {
                    report_spam: false,
                    add_contact: false,
                    block_contact: false,
                    share_contact: false,
                    need_contacts_exception: false,
                    report_geo: false,
                    autoarchived: false,
                    invite_members: false,
                    request_chat_broadcast: false,
                    geo_distance: None,
                    request_chat_title: None,
                    request_chat_date: None,
                },
                personal_photo: None,
                profile_photo: None,
                fallback_photo: None,
                notify_settings: PeerNotifySettings {
                    show_previews: None,
                    silent: None,
                    mute_until: None,
                    ios_sound: None,
                    android_sound: None,
                    other_sound: None,
                },
                bot_info: None,
                pinned_msg_id: None,
                common_chats_count: 0,
                folder_id: None,
                ttl_period: None,
                theme_emoticon: None,
                private_forward_name: None,
                bot_group_admin_rights: None,
                bot_broadcast_admin_rights: None,
                premium_gifts: None,
                wallpaper: None,
            },
        ))
    }

    #[allow(dead_code)]
    pub async fn close(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.transport.close().await?;
        self.closed = true;
        Ok(())
    }
}
