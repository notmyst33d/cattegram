use std::future::Future;
use std::error::Error;
use std::pin::Pin;

pub type AsyncResult<'a, T> = Pin<Box<dyn Future<Output = Result<T, Box<dyn Error>>> + Send + 'a>>;
