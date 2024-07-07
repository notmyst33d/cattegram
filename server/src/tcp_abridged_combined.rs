use crate::transport::Transport;
use crate::Aes256Ctr;
use aes::cipher::StreamCipher;
use async_trait::async_trait;
use std::error::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
pub struct TcpAbridgedCombined {
    socket: TcpStream,
    encrypt: Option<Aes256Ctr>,
    decrypt: Option<Aes256Ctr>,
}

impl TcpAbridgedCombined {
    pub fn new(socket: TcpStream, encrypt: Option<Aes256Ctr>, decrypt: Option<Aes256Ctr>) -> Self {
        Self {
            socket,
            encrypt,
            decrypt,
        }
    }
}

#[async_trait]
impl Transport for TcpAbridgedCombined {
    async fn read(&mut self) -> Result<(Vec<u8>, bool), Box<dyn Error + Send + Sync>> {
        let mut buf = vec![0];
        self.socket.read_exact(&mut buf[..1]).await?;
    
        self.encrypt
            .as_mut()
            .map(|c| c.apply_keystream(&mut buf[..1]));

        let (length, quick_ack) = if buf[0] == 0x7f {
            // Extended length
            let mut lbuf = [0u8; 4];
            self.socket.read_exact(&mut lbuf[..3]).await?;

            self.encrypt
                .as_mut()
                .map(|c| c.apply_keystream(&mut lbuf[..3]));

            ((u32::from_le_bytes(lbuf) as usize) * 4, false)
        } else if buf[0] == 0xff {
            // Extended length + Quick ACK
            let mut lbuf = [0u8; 4];
            self.socket.read_exact(&mut lbuf[..3]).await?;

            self.encrypt
                .as_mut()
                .map(|c| c.apply_keystream(&mut lbuf[..3]));

            ((u32::from_le_bytes(lbuf) as usize) * 4, true)
        } else if buf[0] & (1 << 7) != 0 {
            // Normal length + Quick ACK
            (usize::from(buf[0] ^ (1 << 7)) * 4, true)
        } else {
            // Normal length
            (usize::from(buf[0]) * 4, false)
        };
        buf.resize(length, 0);
        self.socket.read_exact(&mut buf[..length]).await?;

        self.encrypt
            .as_mut()
            .map(|c| c.apply_keystream(&mut buf[..length]));

        Ok((buf, quick_ack))
    }

    async fn write(&mut self, data: &[u8]) -> Result<(), Box<dyn Error + Send + Sync>> {
        let length = if data.len() / 4 >= 0x7f {
            let mut b = (data.len() as u32 / 4).to_le_bytes();
            b.rotate_right(1);
            b[0] = 0x7f;
            b.to_vec()
        } else {
            [(data.len() / 4) as u8].to_vec()
        };
        let mut encrypted_data = [length, data.to_vec()].concat();

        self.decrypt
            .as_mut()
            .map(|c| c.apply_keystream(&mut encrypted_data));

        self.socket.write(&encrypted_data).await?;
        Ok(())
    }

    async fn write_raw(&mut self, data: &[u8]) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut encrypted_data = data.to_vec();

        self.decrypt
            .as_mut()
            .map(|c| c.apply_keystream(&mut encrypted_data));

        self.socket.write(&encrypted_data).await?;
        Ok(())
    }

    async fn close(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.socket.shutdown().await?;
        Ok(())
    }
}
