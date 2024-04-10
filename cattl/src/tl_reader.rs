use crate::{TlObject, TlError, TlReaderFunction, BytesBuffer, mtproto};

pub struct TlReader {
    readers: Vec<(i32, TlReaderFunction)>,
}

impl TlReader {
    pub fn new() -> Self {
        let mut s = Self { readers: vec![] };
        mtproto::extend_reader(&mut s);
        s
    }

    pub fn read(&self, data: &mut BytesBuffer) -> Result<Box<dyn TlObject>, TlError> {
        let hash = data.read_int()?;

        for reader in &self.readers {
            if reader.0 == hash {
                return reader.1(data);
            }
        }

        Err(TlError::NoReader)
    }

    pub fn add_reader(&mut self, hash: i32, reader: TlReaderFunction) {
        self.readers.push((hash, reader));
    }
}
