use crate::session::Session;
use crate::{ok, rpc, v};
use catte_tl_schema::*;
use std::{error::Error, sync::Arc};
use tokio::sync::Mutex;

pub async fn rpc_auth_send_code(
    session: Arc<Mutex<Session>>,
    message: rpc::Message<AuthSendCode>,
) -> Result<SchemaObject, Box<dyn Error + Send + Sync>> {
    let mut locked_session = session.lock().await;

    locked_session.login_flow.phone_number = message.obj.phone_number;
    locked_session.login_flow.code = "12345".to_string(); // TODO: Generate random code

    ok!(
        message,
        AuthSentCode {
            r#type: v!(AuthSentCodeTypeVariant::AuthSentCodeTypeSms { length: 5 }),
            phone_code_hash: String::new(),
            next_type: None,
            timeout: None,
        }
    )
}

pub async fn rpc_auth_sign_in(
    session: Arc<Mutex<Session>>,
    message: rpc::Message<AuthSignIn>,
) -> Result<SchemaObject, Box<dyn Error + Send + Sync>> {
    ok!(
        message,
        AuthAuthorization {
            setup_password_required: false,
            otherwise_relogin_days: None,
            tmp_sessions: None,
            future_auth_token: None,
            user: UserVariant::User(Box::new(session.lock().await.get_user())),
        }
    )
}
