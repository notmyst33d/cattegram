use crate::{Message, PeerUser, PeerVariant, User};

impl Default for User {
    fn default() -> Self {
        Self {
            is_self: Default::default(),
            contact: Default::default(),
            mutual_contact: Default::default(),
            deleted: Default::default(),
            bot: Default::default(),
            bot_chat_history: Default::default(),
            bot_nochats: Default::default(),
            verified: Default::default(),
            restricted: Default::default(),
            min: Default::default(),
            bot_inline_geo: Default::default(),
            support: Default::default(),
            scam: Default::default(),
            apply_min_photo: Default::default(),
            fake: Default::default(),
            bot_attach_menu: Default::default(),
            premium: Default::default(),
            attach_menu_enabled: Default::default(),
            bot_can_edit: Default::default(),
            id: Default::default(),
            access_hash: Default::default(),
            first_name: Default::default(),
            last_name: Default::default(),
            username: Default::default(),
            phone: Default::default(),
            photo: Default::default(),
            status: Default::default(),
            bot_info_version: Default::default(),
            restriction_reason: Default::default(),
            bot_inline_placeholder: Default::default(),
            lang_code: Default::default(),
            emoji_status: Default::default(),
            usernames: Default::default(),
        }
    }
}

impl Default for Message {
    fn default() -> Self {
        Self {
            out: Default::default(),
            mentioned: Default::default(),
            media_unread: Default::default(),
            silent: Default::default(),
            post: Default::default(),
            from_scheduled: Default::default(),
            legacy: Default::default(),
            edit_hide: Default::default(),
            pinned: Default::default(),
            noforwards: Default::default(),
            id: Default::default(),
            from_id: Default::default(),
            peer_id: PeerVariant::PeerUser(Box::new(PeerUser { user_id: 0 })),
            fwd_from: Default::default(),
            via_bot_id: Default::default(),
            reply_to: Default::default(),
            date: Default::default(),
            message: Default::default(),
            media: Default::default(),
            reply_markup: Default::default(),
            entities: Default::default(),
            views: Default::default(),
            forwards: Default::default(),
            replies: Default::default(),
            edit_date: Default::default(),
            post_author: Default::default(),
            grouped_id: Default::default(),
            reactions: Default::default(),
            restriction_reason: Default::default(),
            ttl_period: Default::default(),
        }
    }
}
