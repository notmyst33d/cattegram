use crate::session::Session;
use crate::{err, ok, ok_vec, rpc};
use catte_server::auth;
use catte_tl_schema::*;
use std::{error::Error, sync::Arc};
use tokio::sync::Mutex;

///
/// # Layer 158
/// ## users.getFullUser#b60f5918 id:InputUser = users.UserFull;
/// Returns extended user info by ID.
///
/// ## Parameters
/// | Name | Type | Description |
/// | ---- | ---- | ----------- |
/// | id | InputUser | User ID |
///
/// ## Behavior
/// <strong>❌ This function does not behave the same way as official Telegram servers</strong>
/// * No access_hash handling
/// * Limited user types: self
///
#[auth]
pub async fn rpc_users_get_full_user(
    session: Arc<Mutex<Session>>,
    message: rpc::Message<UsersGetFullUser>,
) -> Result<SchemaObject, Box<dyn Error + Send + Sync>> {
    let session = session.lock().await;
    let (mut self_user, self_user_full) = session.get_self_full().await?;

    match message.obj.id {
        InputUserVariant::InputUserSelf(_) => self_user.is_self = true,
        InputUserVariant::InputUser(input_user) => {
            if input_user.user_id == self_user.id {
                self_user.is_self = true;
            } else {
                todo!();
            }
        }
        _ => todo!(),
    }

    ok!(
        message,
        UsersUserFull {
            full_user: self_user_full,
            chats: vec![],
            users: vec![UserVariant::User(Box::new(self_user))],
        }
    )
}

///
/// # Layer 158
/// ## users.getUsers#d91a548 id:Vector\<InputUser\> = Vector\<User\>;
/// Returns extended user info by ID.
///
/// ## Parameters
/// | Name | Type | Description |
/// | ---- | ---- | ----------- |
/// | id | InputUser | User ID |
///
/// ## Behavior
/// <strong>❌ This function does not behave the same way as official Telegram servers</strong>
/// * No access_hash handling
/// * Limited user types: self, user
///
#[auth]
pub async fn rpc_users_get_users(
    session: Arc<Mutex<Session>>,
    message: rpc::Message<UsersGetUsers>,
) -> Result<SchemaObject, Box<dyn Error + Send + Sync>> {
    let session = session.lock().await;
    let self_user = session.get_self().await?;

    let user_ids = message
        .obj
        .id
        .iter()
        .map(|x| match x {
            InputUserVariant::InputUserSelf(_) => self_user.id,
            InputUserVariant::InputUser(user) => user.user_id,
            _ => todo!(),
        })
        .collect::<Vec<i64>>();

    ok_vec!(
        message,
        session
            .storage
            .get_users(&user_ids)
            .await?
            .into_iter()
            .map(|mut x| {
                if x.id == self_user.id {
                    x.is_self = true
                }
                SchemaObject::User(x)
            })
            .collect()
    )
}
