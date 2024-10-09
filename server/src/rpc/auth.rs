use crate::session::Session;
use crate::{err, ok, rpc, v};
use catte_tl_schema::*;
use std::{error::Error, sync::Arc};
use tokio::sync::Mutex;

pub async fn rpc_auth_send_code(
    session: Arc<Mutex<Session>>,
    message: rpc::Message<AuthSendCode>,
) -> Result<SchemaObject, Box<dyn Error + Send + Sync>> {
    let mut session = session.lock().await;

    if session.authorized {
        err!(message, 400, "PHONE_NUMBER_FLOOD");
    }

    session.login_flow.phone_number = message.obj.phone_number;
    session.login_flow.code = "12345".to_string(); // TODO: Generate random code

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
    let mut session = session.lock().await;

    if session.authorized {
        err!(message, 500, "SIGN_IN_FAILED")
    }

    if let Some(code) = message.obj.phone_code {
        if code != session.login_flow.code {
            err!(message, 400, "PHONE_CODE_INVALID")
        }
    } else {
        err!(message, 400, "PHONE_CODE_EMPTY")
    }

    session.login_flow.phone_number_verified = true;

    if let Ok(mut user) = session
        .storage
        .get_user_by_phone(&session.login_flow.phone_number)
        .await
    {
        user.is_self = true;
        session
            .storage
            .insert_session(session.auth_key_id, user.id)
            .await?;
        session.authorized = true;
        ok!(
            message,
            AuthAuthorization {
                setup_password_required: false,
                otherwise_relogin_days: None,
                tmp_sessions: None,
                future_auth_token: None,
                user: UserVariant::User(Box::new(user)),
            }
        )
    }

    ok!(
        message,
        AuthAuthorizationSignUpRequired {
            terms_of_service: None
        }
    )
}

pub async fn rpc_auth_sign_up(
    session: Arc<Mutex<Session>>,
    message: rpc::Message<AuthSignUp>,
) -> Result<SchemaObject, Box<dyn Error + Send + Sync>> {
    let mut session = session.lock().await;

    if session.authorized {
        err!(message, 500, "PHONE_NUMBER_FLOOD");
    }

    if !session.login_flow.phone_number_verified {
        err!(message, 406, "PHONE_NUMBER_INVALID");
    }

    let user = session
        .storage
        .insert_user(
            &message.obj.first_name,
            &message.obj.last_name,
            &session.login_flow.phone_number,
        )
        .await?;

    session
        .storage
        .insert_session(session.auth_key_id, user.id)
        .await?;
    session.authorized = true;

    ok!(
        message,
        AuthAuthorization {
            setup_password_required: false,
            otherwise_relogin_days: None,
            tmp_sessions: None,
            future_auth_token: None,
            user: UserVariant::User(Box::new(user)),
        }
    )
}
