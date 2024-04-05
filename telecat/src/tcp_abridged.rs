use std::error::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use crate::transport::Transport;
use crate::types::AsyncResult;
use async_trait::async_trait;

pub struct TcpAbridged {
    socket: TcpStream,
}

impl TcpAbridged {
    pub fn new(socket: TcpStream) -> Self {
        Self { socket }
    }
}

#[async_trait]
impl Transport for TcpAbridged {
    async fn read(&mut self) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut buf = vec![0];
        self.socket.read_exact(&mut buf[..1]).await?;
        let mut length = buf[0] as usize;
        if length == 0x7f {
            todo!();
        } else {
            length = length * 4;
        }
        buf.resize(length, 0);
        self.socket.read_exact(&mut buf[..length]).await?;
        Ok(buf)
    }

    async fn write(&mut self, data: &[u8]) -> Result<(), Box<dyn Error>> {
        self.socket.write(&[(data.len() / 4) as u8]).await?;
        self.socket.write(data).await?;
        Ok(())
    }

    async fn close(&mut self) -> Result<(), Box<dyn Error>> {
        self.socket.shutdown().await?;
        Ok(())
    }
}
