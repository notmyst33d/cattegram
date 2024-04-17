use std::error::Error;
use cattl::{TlObject, TlReader, BytesBuffer};
use crate::transport::Transport;

pub enum SessionState {
    AuthKey,
}

#[derive(Default)]
pub struct AuthKeyFlow {
    pub nonce: i128,
    pub server_nonce: i128,
    pub p: u32,
    pub q: u32,
}

pub struct Session {
    transport: Box<dyn Transport + Send + Sync>,
    state: SessionState,
    pub auth_key_flow: AuthKeyFlow,
    reader: TlReader,
}

impl Session {
    pub fn new(transport: Box<dyn Transport + Send + Sync>) -> Self {
        Self { transport, state: SessionState::AuthKey, auth_key_flow: AuthKeyFlow::default(), reader: TlReader::new() }
    }

    pub async fn receive_raw(&mut self) -> Result<Box<dyn TlObject + Send + Sync>, Box<dyn Error>> {
        let mut data = BytesBuffer::new(self.transport.read().await?);
        data.seek(20);
        #[cfg(debug_assertions)] {
            let all = data.data();
            let datahex = all.into_iter().map(|v| format!("{:02x?}", v)).collect::<Vec<_>>().join("");
            println!("trace: client -> server: {}", datahex);
        }
        Ok(self.reader.read(&mut data)?)
    }

    pub async fn send_raw(&mut self, obj: Box<dyn TlObject + Send + Sync>) -> Result<(), Box<dyn Error>> {
        let mut data = BytesBuffer::new(vec![0; 20]);
        data.seek(20);
        obj.write(&mut data);
        #[cfg(debug_assertions)] {
            let all = data.data();
            let datahex = all.into_iter().map(|v| format!("{:02x?}", v)).collect::<Vec<_>>().join("");
            println!("trace: server -> client: {}", datahex);
        }
        self.transport.write(&data.data()).await?;
        Ok(())
    }

    pub async fn close(&mut self) -> Result<(), Box<dyn Error>> {
        self.transport.close().await?;
        Ok(())
    }
}
