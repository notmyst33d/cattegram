use async_trait::async_trait;
use std::error::Error;

#[async_trait]
pub trait Transport: Send + Sync {
    async fn read(&mut self) -> Result<(Vec<u8>, bool), Box<dyn Error + Send + Sync>>;
    async fn write(&mut self, data: &[u8]) -> Result<(), Box<dyn Error + Send + Sync>>;
    async fn write_raw(&mut self, data: &[u8]) -> Result<(), Box<dyn Error + Send + Sync>>;
    async fn close(&mut self) -> Result<(), Box<dyn Error + Send + Sync>>;
}
