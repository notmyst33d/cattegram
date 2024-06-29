use crate::transport::Transport;
use cattl::{BytesBuffer, TlObject, TlReader};
use num_bigint::BigUint;
use std::error::Error;

pub enum SessionState {
    AuthKey,
}

pub struct AuthKeyFlow {
    pub nonce: i128,
    pub server_nonce: i128,
    pub new_nonce: [u8; 32],
    pub tmp_aes_key: [u8; 32],
    pub tmp_aes_iv: [u8; 32],
    pub p: u32,
    pub q: u32,
    pub a: BigUint,
    pub g: i32,
    pub g_a: BigUint,
}

impl AuthKeyFlow {
    pub fn new() -> Self {
        Self {
            nonce: 0,
            server_nonce: 0,
            new_nonce: [0u8; 32],
            tmp_aes_key: [0u8; 32],
            tmp_aes_iv: [0u8; 32],
            p: 0,
            q: 0,
            a: BigUint::default(),
            g: 3, // TODO: Does this ever change?
            g_a: BigUint::default(),
        }
    }
}

pub struct Session {
    pub state: SessionState,
    pub auth_key_flow: AuthKeyFlow,
    pub auth_key_id: [u8; 8],
    pub auth_key: [u8; 256],
    transport: Box<dyn Transport + Send + Sync>,
    reader: TlReader,
}

impl Session {
    pub fn new(transport: Box<dyn Transport + Send + Sync>) -> Self {
        Self {
            state: SessionState::AuthKey,
            auth_key_flow: AuthKeyFlow::new(),
            auth_key_id: [0u8; 8],
            auth_key: [0u8; 256],
            transport,
            reader: TlReader::new(),
        }
    }

    pub async fn receive_raw(&mut self) -> Result<Box<dyn TlObject + Send + Sync>, Box<dyn Error>> {
        let mut data = BytesBuffer::new(self.transport.read().await?);
        data.seek(20);
        #[cfg(no_debug_assertions)]
        {
            let all = data.data();
            let datahex = all
                .into_iter()
                .map(|v| format!("{:02x?}", v))
                .collect::<Vec<_>>()
                .join("");
            println!("trace: client -> server: {}", datahex);
        }
        Ok(self.reader.read(&mut data)?)
    }

    pub async fn send_raw(
        &mut self,
        obj: Box<dyn TlObject + Send + Sync>,
    ) -> Result<(), Box<dyn Error>> {
        let mut data = BytesBuffer::new(vec![0; 20]);
        data.seek(20);
        obj.write(&mut data);
        #[cfg(no_debug_assertions)]
        {
            let all = data.data();
            let datahex = all
                .into_iter()
                .map(|v| format!("{:02x?}", v))
                .collect::<Vec<_>>()
                .join("");
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
