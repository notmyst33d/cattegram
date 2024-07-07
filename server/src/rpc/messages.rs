use crate::session::Session;
use crate::{ok, ok_vec, rpc, time, v};
use catte_tl_schema::*;
use std::{error::Error, sync::Arc};
use tokio::sync::Mutex;

pub async fn rpc_messages_get_featured_stickers(
    _session: Arc<Mutex<Session>>,
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

pub async fn rpc_messages_get_featured_emoji_stickers(
    _session: Arc<Mutex<Session>>,
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

pub async fn rpc_messages_get_sticker_set(
    _session: Arc<Mutex<Session>>,
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

pub async fn rpc_messages_get_dialogs(
    _session: Arc<Mutex<Session>>,
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

pub async fn rpc_messages_get_history(
    session: Arc<Mutex<Session>>,
    message: rpc::Message<MessagesGetHistory>,
) -> Result<SchemaObject, Box<dyn Error + Send + Sync>> {
    let mut locked_session = session.lock().await;
    let user_id = locked_session.get_user_id();
    let mut messages = vec![];

    match message.obj.peer {
        InputPeerVariant::InputPeerEmpty(_) => todo!(),
        InputPeerVariant::InputPeerSelf(_) => {
            for message in locked_session.storage.get_user_messages(
                user_id,
                user_id,
                message.obj.offset_id,
                message.obj.limit,
            )? {
                messages.push(v!(MessageVariant::Message {
                    out: true,
                    mentioned: false,
                    media_unread: false,
                    silent: false,
                    post: false,
                    from_scheduled: false,
                    legacy: false,
                    edit_hide: true,
                    pinned: false,
                    noforwards: false,
                    id: message.id,
                    from_id: None,
                    peer_id: v!(PeerVariant::PeerUser {
                        user_id: message.chat_id
                    }),
                    fwd_from: None,
                    via_bot_id: None,
                    reply_to: None,
                    date: message.date,
                    message: message.text,
                    media: None,
                    reply_markup: None,
                    entities: None,
                    views: None,
                    forwards: None,
                    replies: None,
                    edit_date: None,
                    post_author: None,
                    grouped_id: None,
                    reactions: None,
                    restriction_reason: None,
                    ttl_period: None
                }))
            }
        }
        InputPeerVariant::InputPeerChat(_) => todo!(),
        InputPeerVariant::InputPeerUser(peer) => {
            for message in locked_session.storage.get_user_messages(
                peer.user_id,
                peer.user_id,
                message.obj.offset_id,
                message.obj.limit,
            )? {
                messages.push(v!(MessageVariant::Message {
                    out: true,
                    mentioned: false,
                    media_unread: false,
                    silent: false,
                    post: false,
                    from_scheduled: false,
                    legacy: false,
                    edit_hide: true,
                    pinned: false,
                    noforwards: false,
                    id: message.id,
                    from_id: None,
                    peer_id: v!(PeerVariant::PeerUser {
                        user_id: message.chat_id
                    }),
                    fwd_from: None,
                    via_bot_id: None,
                    reply_to: None,
                    date: message.date,
                    message: message.text,
                    media: None,
                    reply_markup: None,
                    entities: None,
                    views: None,
                    forwards: None,
                    replies: None,
                    edit_date: None,
                    post_author: None,
                    grouped_id: None,
                    reactions: None,
                    restriction_reason: None,
                    ttl_period: None
                }))
            }
        }
        InputPeerVariant::InputPeerChannel(_) => todo!(),
        InputPeerVariant::InputPeerUserFromMessage(_) => todo!(),
        InputPeerVariant::InputPeerChannelFromMessage(_) => todo!(),
    }

    ok!(
        message,
        MessagesMessages {
            messages: messages,
            chats: vec![],
            users: vec![UserVariant::User(Box::new(locked_session.get_user()))],
        }
    )
}

pub async fn rpc_messages_send_message(
    session: Arc<Mutex<Session>>,
    message: rpc::Message<MessagesSendMessage>,
) -> Result<SchemaObject, Box<dyn Error + Send + Sync>> {
    let mut locked_session = session.lock().await;
    let user_id = locked_session.get_user_id();

    let message_id = match message.obj.peer {
        InputPeerVariant::InputPeerEmpty(_) => todo!(),
        InputPeerVariant::InputPeerSelf(_) => locked_session.storage.store_user_message(
            user_id,
            user_id,
            message.obj.message.clone(),
        )?,
        InputPeerVariant::InputPeerChat(_) => todo!(),
        InputPeerVariant::InputPeerUser(peer) => locked_session.storage.store_user_message(
            user_id,
            peer.user_id,
            message.obj.message.clone(),
        )?,
        InputPeerVariant::InputPeerChannel(_) => todo!(),
        InputPeerVariant::InputPeerUserFromMessage(_) => todo!(),
        InputPeerVariant::InputPeerChannelFromMessage(_) => todo!(),
    };

    ok!(
        message,
        UpdateShortSentMessage {
            out: true,
            id: message_id,
            pts: 0,
            pts_count: 0,
            date: time!(),
            entities: None,
            ttl_period: None,
            media: None,
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
