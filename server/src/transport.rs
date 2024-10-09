use async_trait::async_trait;

#[async_trait]
pub trait Transport: Send + Sync {
    async fn read(&mut self) -> Result<(Vec<u8>, bool), std::io::Error>;
    async fn write(&mut self, data: &[u8]) -> Result<(), std::io::Error>;
    async fn write_quick_ack(&mut self, ack_token: u32) -> Result<(), std::io::Error>;
    async fn close(&mut self) -> Result<(), std::io::Error>;
}
