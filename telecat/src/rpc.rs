use std::error::Error;
use std::sync::Arc;
use tokio::sync::Mutex;
use futures::future::BoxFuture;
use cattl::TlObject;
use crate::session::Session;

pub type RpcResult = Result<Box<dyn TlObject + Send + Sync>, Box<dyn Error>>;
pub type RpcFunction = for<'a> fn(Arc<Mutex<Session>>, Box<dyn TlObject + Send + Sync + 'a>) -> BoxFuture<'a, RpcResult>;
pub type RpcMapping = Vec<(i32, RpcFunction)>;
