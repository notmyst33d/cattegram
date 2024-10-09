use crate::session::Session;
use crate::{ok_vec, rpc};
use catte_tl_schema::*;
use std::{error::Error, sync::Arc};
use tokio::sync::Mutex;

///
/// # Undocumented Layer
/// ## langpack.getLanguages#800fd57d = Vector\<LangPackLanguage\>;
/// Get information about all languages in a localization pack
///
/// ## Parameters
/// None
///
/// ## Behavior
/// <strong>❌ This function does not behave the same way as official Telegram servers</strong>
/// * Stub
///
pub async fn rpc_langpack_get_languages(
    _session: Arc<Mutex<Session>>,
    message: rpc::Message<LangpackGetLanguages>,
) -> Result<SchemaObject, Box<dyn Error + Send + Sync>> {
    ok_vec!(message, vec![])
}
