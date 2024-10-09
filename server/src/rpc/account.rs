use crate::session::Session;
use crate::{err, ok_user, rpc};
use catte_tl_schema::*;
use std::{error::Error, sync::Arc};
use tokio::sync::Mutex;

pub async fn rpc_account_update_username(
    session: Arc<Mutex<Session>>,
    message: rpc::Message<AccountUpdateUsername>,
) -> Result<SchemaObject, Box<dyn Error + Send + Sync>> {
    let session = session.lock().await;
    let self_user = session.get_self().await?;

    if let Ok(_) = session
        .storage
        .get_user_by_username(&message.obj.username)
        .await
    {
        err!(message, 400, "USERNAME_OCCUPIED")
    }

    session
        .storage
        .update_username(self_user.id, &message.obj.username)
        .await?;

    ok_user!(message, session.get_self().await?)
}
