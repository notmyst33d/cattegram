use crate::session::Session;
use crate::{ok, rpc, time};
use catte_tl_schema::*;
use std::{error::Error, sync::Arc};
use tokio::sync::Mutex;

pub async fn rpc_updates_get_state(
    _session: Arc<Mutex<Session>>,
    message: rpc::Message<UpdatesGetState>,
) -> Result<SchemaObject, Box<dyn Error + Send + Sync>> {
    ok!(
        message,
        UpdatesState {
            pts: 0,
            qts: 0,
            date: time!(),
            seq: 0,
            unread_count: 0,
        }
    )
}
