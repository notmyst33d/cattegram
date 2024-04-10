use std::error::Error;
use cattl::{TlObject, TlReader, BytesBuffer};
use crate::transport::Transport;

enum SessionState {
    AuthKey,
}

pub struct Session {
    transport: Box<dyn Transport + Send + Sync>,
    state: SessionState,
    reader: TlReader,
}

impl Session {
    pub fn new(transport: Box<dyn Transport + Send + Sync>) -> Self {
        Self { transport, state: SessionState::AuthKey, reader: TlReader::new() }
    }

    pub async fn receive_raw(&mut self) -> Result<Box<dyn TlObject + Send + Sync>, Box<dyn Error>> {
        let mut data = BytesBuffer::new(self.transport.read().await?);
        data.seek(20);
        Ok(self.reader.read(&mut data)?)
    }

    pub async fn send_raw(&mut self, obj: Box<dyn TlObject + Send + Sync>) -> Result<(), Box<dyn Error>> {
        let mut data = BytesBuffer::new(vec![0; 20]);
        data.seek(20);
        obj.write(&mut data);
        self.transport.write(&data.data()).await?;
        Ok(())
    }

    pub async fn close(&mut self) -> Result<(), Box<dyn Error>> {
        self.transport.close().await?;
        Ok(())
    }
}
