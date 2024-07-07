use crate::session::Session;
use crate::{ok, ok_vec, rpc};
use catte_tl_schema::*;
use std::{error::Error, sync::Arc};
use tokio::sync::Mutex;

pub async fn rpc_users_get_full_user(
    session: Arc<Mutex<Session>>,
    message: rpc::Message<UsersGetFullUser>,
) -> Result<SchemaObject, Box<dyn Error + Send + Sync>> {
    let locked_session = session.lock().await;
    ok!(
        message,
        UsersUserFull {
            full_user: locked_session.get_user_full(),
            chats: vec![],
            users: vec![UserVariant::User(Box::new(locked_session.get_user()))],
        }
    )
}

pub async fn rpc_users_get_users(
    session: Arc<Mutex<Session>>,
    message: rpc::Message<UsersGetUsers>,
) -> Result<SchemaObject, Box<dyn Error + Send + Sync>> {
    let locked_session = session.lock().await;
    ok_vec!(message, vec![SchemaObject::User(locked_session.get_user())])
}
