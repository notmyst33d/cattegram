use crate::session::Session;
use crate::{ok_vec, rpc};
use catte_tl_schema::*;
use std::{error::Error, sync::Arc};
use tokio::sync::Mutex;

pub async fn rpc_langpack_get_languages(
    _session: Arc<Mutex<Session>>,
    message: rpc::Message<LangpackGetLanguages>,
) -> Result<SchemaObject, Box<dyn Error + Send + Sync>> {
    ok_vec!(message, vec![])
}
