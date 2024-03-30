use std::error::Error;
use async_trait::async_trait;

#[async_trait]
pub trait Transport {
    async fn read(&mut self) -> Result<Vec<u8>, Box<dyn Error>>;
    async fn write(&mut self, data: &[u8]) -> Result<(), Box<dyn Error>>;
    async fn close(&mut self) -> Result<(), Box<dyn Error>>;
}
