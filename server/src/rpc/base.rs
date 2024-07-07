use crate::session::Session;
use crate::{ok, ok_raw, rpc};
use catte_tl_schema::*;
use std::{error::Error, sync::Arc};
use tokio::sync::Mutex;

pub async fn rpc_destroy_session(
    _session: Arc<Mutex<Session>>,
    message: rpc::Message<DestroySession>,
) -> Result<SchemaObject, Box<dyn Error + Send + Sync>> {
    ok!(
        message,
        DestroySessionOk {
            session_id: message.obj.session_id
        }
    )
}

pub async fn rpc_ping(
    _session: Arc<Mutex<Session>>,
    message: rpc::Message<Ping>,
) -> Result<SchemaObject, Box<dyn Error + Send + Sync>> {
    ok_raw!(Pong {
        msg_id: message.msg_id,
        ping_id: message.obj.ping_id,
    })
}

pub async fn rpc_ping_delay_disconnect(
    _session: Arc<Mutex<Session>>,
    message: rpc::Message<PingDelayDisconnect>,
) -> Result<SchemaObject, Box<dyn Error + Send + Sync>> {
    ok_raw!(Pong {
        msg_id: message.msg_id,
        ping_id: message.obj.ping_id,
    })
}

pub async fn rpc_invoke_with_layer(
    session: Arc<Mutex<Session>>,
    message: rpc::Message<InvokeWithLayer>,
) -> Result<SchemaObject, Box<dyn Error + Send + Sync>> {
    Box::pin(async {
        rpc::invoke(
            session,
            (message.msg_id, message.seq_no, *message.obj.query),
        )
        .await
    })
    .await
}

pub async fn rpc_invoke_after_msg(
    session: Arc<Mutex<Session>>,
    message: rpc::Message<InvokeAfterMsg>,
) -> Result<SchemaObject, Box<dyn Error + Send + Sync>> {
    Box::pin(async {
        rpc::invoke(
            session,
            (message.msg_id, message.seq_no, *message.obj.query),
        )
        .await
    })
    .await
}

pub async fn rpc_init_connection(
    session: Arc<Mutex<Session>>,
    message: rpc::Message<InitConnection>,
) -> Result<SchemaObject, Box<dyn Error + Send + Sync>> {
    Box::pin(async {
        rpc::invoke(
            session,
            (message.msg_id, message.seq_no, *message.obj.query),
        )
        .await
    })
    .await
}
