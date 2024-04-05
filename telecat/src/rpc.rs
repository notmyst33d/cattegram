use std::any::Any;
use std::sync::Arc;
use tokio::sync::Mutex;
use futures::future::BoxFuture;
use cattl::TlObject;
use crate::session::Session;

pub type RpcResult = Result<Box<dyn TlObject>, Box<dyn Any>>;
pub type RpcFunction = for <'a> fn(Arc<Mutex<Session>>, &'a (dyn TlObject + Sync)) -> BoxFuture<'a, RpcResult>;
pub type RpcMapping = Vec<(i32, RpcFunction)>;
