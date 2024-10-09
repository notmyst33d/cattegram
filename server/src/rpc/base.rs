use crate::session::Session;
use crate::{ok, ok_raw, rpc};
use catte_tl_schema::*;
use std::{error::Error, sync::Arc};
use tokio::sync::Mutex;

///
/// # MTProto Layer
/// ## destroy_session#e7512126 session_id:long = DestroySessionRes;
/// Destroys the session.
///
/// ## Parameters
/// | Name | Type | Description |
/// | ---- | ---- | ----------- |
/// | session_id | long | Session ID |
///
/// ## Behavior
/// <strong>⚠️ This function behaves the same way as official Telegram servers, but has some quirks</strong>
/// * Doesnt destroy the session
///
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

///
/// # MTProto Layer
/// ## ping#7abe77ec ping_id:long = Pong;
/// Responds to a ping with a certain msg_id.
///
/// ## Parameters
/// | Name | Type | Description |
/// | ---- | ---- | ----------- |
/// | ping_id | long | Ping ID |
///
/// ## Behavior
/// <strong>✅ This function behaves the same way as official Telegram servers</strong>
///
pub async fn rpc_ping(
    _session: Arc<Mutex<Session>>,
    message: rpc::Message<Ping>,
) -> Result<SchemaObject, Box<dyn Error + Send + Sync>> {
    ok_raw!(Pong {
        msg_id: message.msg_id,
        ping_id: message.obj.ping_id,
    })
}

///
/// # MTProto Layer
/// ## ping_delay_disconnect#f3427b8c ping_id:long disconnect_delay:int = Pong;
/// Responds to a ping with a certain msg_id and disconnects after a given delay.
///
/// ## Parameters
/// | Name | Type | Description |
/// | ---- | ---- | ----------- |
/// | ping_id | long | Ping ID |
/// | disconnect_delay | int | Delay amount in seconds after which the server should disconnect |
///
/// ## Behavior
/// <strong>⚠️ This function behaves the same way as official Telegram servers, but has some quirks</strong>
/// * Doesnt disconnect after a given delay
///
pub async fn rpc_ping_delay_disconnect(
    _session: Arc<Mutex<Session>>,
    message: rpc::Message<PingDelayDisconnect>,
) -> Result<SchemaObject, Box<dyn Error + Send + Sync>> {
    ok_raw!(Pong {
        msg_id: message.msg_id,
        ping_id: message.obj.ping_id,
    })
}

///
/// # Layer 158
/// ## invokeWithLayer#da9b0d0d {X:Type} layer:int query:!X = X;
/// Invoke the specified query using the specified API layer
///
/// ## Parameters
/// | Name | Type | Description |
/// | ---- | ---- | ----------- |
/// | layer | long | The layer to use |
/// | query | !X | The query |
///
/// ## Behavior
/// <strong>✅ This function behaves the same way as official Telegram servers</strong>
///
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

///
/// # Layer 158
/// ## invokeAfterMsg#cb9f372d {X:Type} msg_id:long query:!X = X;
/// Invokes a query after successful completion of one of the previous queries.
///
/// ## Parameters
/// | Name | Type | Description |
/// | ---- | ---- | ----------- |
/// | msg_id | long | Message identifier on which a current query depends |
/// | query | !X | The query itself |
///
/// ## Behavior
/// <strong>❌ This function does not behave the same way as official Telegram servers</strong>
/// * Immediately executes the query without checking if msg_id was successfully completed
///
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

///
/// # Layer 158
/// ## initConnection#c1cd5ea9 {X:Type} flags:# api_id:int device_model:string system_version:string app_version:string system_lang_code:string lang_pack:string lang_code:string proxy:flags.0?InputClientProxy params:flags.1?JSONValue query:!X = X;
/// Initialize connection
///
/// ## Parameters
/// | Name | Type | Description |
/// | ---- | ---- | ----------- |
/// | flags | # | Flags, see TL conditional fields |
/// | api_id | int | Application identifier (see. App configuration) |
/// | device_model | string | Device model |
/// | system_version | string | Operation system version |
/// | app_version | string | Application version |
/// | system_lang_code | string | Code for the language used on the device's OS, ISO 639-1 standard |
/// | lang_pack | string | Language pack to use |
/// | lang_code | string | Code for the language used on the client, ISO 639-1 standard |
/// | proxy | flags.0?InputClientProxy | Info about an MTProto proxy |
/// | params | flags.1?JSONValue | Additional initConnection parameters. For now, only the tz_offset field is supported, for specifying timezone offset in seconds. |
/// | query | !X | The query itself |
///
/// ## Behavior
/// <strong>❌ This function does not behave the same way as official Telegram servers</strong>
/// * Immediately executes the query without checking if msg_id was successfully completed
///
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
