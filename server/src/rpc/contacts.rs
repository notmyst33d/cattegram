use crate::session::Session;
use crate::{err, ok, rpc, v};
use catte_tl_schema::*;
use std::{error::Error, sync::Arc};
use tokio::sync::Mutex;

pub async fn rpc_contacts_resolve_username(
    session: Arc<Mutex<Session>>,
    message: rpc::Message<ContactsResolveUsername>,
) -> Result<SchemaObject, Box<dyn Error + Send + Sync>> {
    let session = session.lock().await;
    let mut user = match session
        .storage
        .get_user_by_username(&message.obj.username)
        .await
    {
        Ok(r) => r,
        Err(_) => err!(message, 400, "USERNAME_NOT_OCCUPIED"),
    };
    user.access_hash = Some(0);
    ok!(
        message,
        ContactsResolvedPeer {
            peer: v!(PeerVariant::PeerUser { user_id: user.id }),
            chats: vec![],
            users: vec![UserVariant::User(Box::new(user))],
        }
    )
}
