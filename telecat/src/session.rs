use std::error::Error;
use cattl::BytesBuffer;
use cattl::TlObject;
use crate::transport::Transport;

enum SessionState {
    AuthKey,
}

pub struct Session {
    state: SessionState,
    transport: Box<dyn Transport + Send + Sync>,
}

impl Session {
    pub fn new(transport: Box<dyn Transport + Send + Sync>) -> Self {
        Self { state: SessionState::AuthKey, transport }
    }

    pub async fn receive_raw<T: 'static>(&mut self) -> Result<T, Box<dyn Error>> {
        let mut data = BytesBuffer::new(self.transport.read().await?);
        data.seek(20);
        match cattl::read(&mut data).unwrap().downcast::<T>() {
            Ok(result) => Ok(*result),
            Err(_) => Err("Downcast failed".into()),
        }
    }

    pub async fn send_raw(&mut self, obj: &(dyn TlObject + Send + Sync)) -> Result<(), Box<dyn Error>> {
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
