use crate::session::Session;
use crate::{err, ok, ok_vec, rpc, time};
use catte_server::auth;
use catte_tl_schema::*;
use std::collections::HashMap;
use std::{error::Error, sync::Arc};
use tokio::sync::Mutex;

#[auth]
pub async fn rpc_messages_get_featured_stickers(
    session: Arc<Mutex<Session>>,
    message: rpc::Message<MessagesGetFeaturedStickers>,
) -> Result<SchemaObject, Box<dyn Error + Send + Sync>> {
    ok!(
        message,
        MessagesFeaturedStickers {
            premium: false,
            hash: message.obj.hash,
            count: 0,
            sets: vec![],
            unread: vec![],
        }
    )
}

#[auth]
pub async fn rpc_messages_get_featured_emoji_stickers(
    session: Arc<Mutex<Session>>,
    message: rpc::Message<MessagesGetFeaturedEmojiStickers>,
) -> Result<SchemaObject, Box<dyn Error + Send + Sync>> {
    ok!(
        message,
        MessagesFeaturedStickers {
            premium: false,
            hash: message.obj.hash,
            count: 0,
            sets: vec![],
            unread: vec![],
        }
    )
}

#[auth]
pub async fn rpc_messages_get_sticker_set(
    session: Arc<Mutex<Session>>,
    message: rpc::Message<MessagesGetStickerSet>,
) -> Result<SchemaObject, Box<dyn Error + Send + Sync>> {
    ok!(
        message,
        MessagesStickerSet {
            set: StickerSet {
                archived: false,
                official: false,
                masks: false,
                animated: false,
                videos: false,
                emojis: false,
                installed_date: None,
                id: 1,
                access_hash: 0,
                title: "empty".to_string(),
                short_name: "empty".to_string(),
                thumbs: None,
                thumb_dc_id: None,
                thumb_version: None,
                thumb_document_id: None,
                count: 0,
                hash: message.obj.hash,
            },
            packs: vec![],
            keywords: vec![],
            documents: vec![],
        }
    )
}

#[auth]
pub async fn rpc_messages_get_dialogs(
    session: Arc<Mutex<Session>>,
    message: rpc::Message<MessagesGetDialogs>,
) -> Result<SchemaObject, Box<dyn Error + Send + Sync>> {
    ok!(
        message,
        MessagesDialogs {
            dialogs: vec![],
            messages: vec![],
            chats: vec![],
            users: vec![],
        }
    )
}

#[auth]
pub async fn rpc_messages_get_history(
    session: Arc<Mutex<Session>>,
    message: rpc::Message<MessagesGetHistory>,
) -> Result<SchemaObject, Box<dyn Error + Send + Sync>> {
    let session = session.lock().await;
    let self_user = session.get_self().await?;

    let (mb_key_primary, mb_key_secondary) = match message.obj.peer {
        InputPeerVariant::InputPeerSelf(_) => (self_user.id, Some(self_user.id)),
        InputPeerVariant::InputPeerUser(peer) => {
            let mut k = vec![peer.user_id, self_user.id];
            k.sort();
            (k[0], Some(k[1]))
        }
        _ => todo!(),
    };

    let messages = session
        .storage
        .get_messages(
            mb_key_primary,
            mb_key_secondary,
            message.obj.limit,
            message.obj.offset_id,
        )
        .await?
        .into_iter()
        .map(|mut m| match m.peer_id {
            PeerVariant::PeerUser(ref peer) => {
                if peer.user_id == self_user.id {
                    m.out = true;
                }
                MessageVariant::Message(Box::new(m))
            }
            _ => todo!(),
        })
        .collect::<Vec<MessageVariant>>();

    let mut users: HashMap<i64, UserVariant> = HashMap::new();
    for message in messages.iter() {
        match message {
            MessageVariant::Message(m) => match &m.peer_id {
                PeerVariant::PeerUser(u) => {
                    if !users.contains_key(&u.user_id) {
                        let mut user = session.storage.get_user(u.user_id).await?;
                        user.access_hash = Some(0);
                        users.insert(u.user_id, UserVariant::User(Box::new(user)));
                    }
                }
                _ => todo!(),
            },
            _ => todo!(),
        }
    }

    let count = messages.len() as i32;
    ok!(
        message,
        MessagesMessagesSlice {
            messages,
            chats: vec![],
            users: users.into_values().collect(),
            inexact: false,
            count,
            next_rate: None,
            offset_id_offset: None,
        }
    )
}

#[auth]
pub async fn rpc_messages_send_message(
    session: Arc<Mutex<Session>>,
    message: rpc::Message<MessagesSendMessage>,
) -> Result<SchemaObject, Box<dyn Error + Send + Sync>> {
    let session = session.lock().await;
    let self_user = session.get_self().await?;

    let (mb_key_primary, mb_key_secondary, peer_id, from_id) = match message.obj.peer {
        InputPeerVariant::InputPeerSelf(_) => {
            (self_user.id, Some(self_user.id), self_user.id, None)
        }
        InputPeerVariant::InputPeerUser(peer) => {
            let mut k = vec![peer.user_id, self_user.id];
            k.sort();
            (k[0], Some(k[1]), self_user.id, None)
        }
        _ => todo!(),
    };

    let date = time!();
    let mut sent_message = session
        .storage
        .insert_message(
            mb_key_primary,
            mb_key_secondary,
            peer_id,
            from_id,
            message.obj.message,
        )
        .await?;

    sent_message.out = true;

    let pts = match session
        .storage
        .increment_mb_pts(mb_key_primary, mb_key_secondary, self_user.id, 1)
        .await
    {
        Ok(r) => r,
        Err(e) => {
            session
                .storage
                .insert_mb_pts(mb_key_primary, mb_key_secondary, self_user.id)
                .await?;
            0
        }
    };

    ok!(
        message,
        Updates {
            updates: vec![UpdateVariant::UpdateNewMessage(Box::new(
                UpdateNewMessage {
                    message: MessageVariant::Message(Box::new(sent_message)),
                    pts,
                    pts_count: 1,
                }
            ))],
            users: vec![UserVariant::User(Box::new(self_user))],
            chats: vec![],
            date,
            seq: 0,
        }
    )
}

pub async fn rpc_messages_get_search_counters(
    _session: Arc<Mutex<Session>>,
    message: rpc::Message<MessagesGetSearchCounters>,
) -> Result<SchemaObject, Box<dyn Error + Send + Sync>> {
    ok_vec!(message, vec![])
}

pub async fn rpc_messages_get_messages_reactions(
    _session: Arc<Mutex<Session>>,
    message: rpc::Message<MessagesGetMessagesReactions>,
) -> Result<SchemaObject, Box<dyn Error + Send + Sync>> {
    ok!(
        message,
        Updates {
            updates: vec![],
            users: vec![],
            chats: vec![],
            date: time!(),
            seq: 0,
        }
    )
}
