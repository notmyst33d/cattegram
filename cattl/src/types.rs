use crate::{TlObject, TlError, BytesBuffer};

pub type TlReaderFunction = fn(&mut BytesBuffer) -> Result<Box<dyn TlObject + Send + Sync>, TlError>;
