use crate::{TlObject, TlError, BytesBuffer};

pub type TlReaderFunction = fn(&mut BytesBuffer) -> Result<Box<dyn TlObject>, TlError>;
