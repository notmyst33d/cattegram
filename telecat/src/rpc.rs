use std::error::Error;
use std::sync::Arc;
use tokio::sync::Mutex;
use futures::future::BoxFuture;
use cattl::TlObject;
use crate::session::Session;

#[macro_export]
macro_rules! impl_rpc {
    ($h:expr, $f:tt, $t:ty) => {
        ($h, |session, obj| {
            Box::pin($f(session, unsafe {
                let (raw, alloc): (*mut dyn Any, _) = Box::into_raw_with_allocator(obj);
                Box::from_raw_in(raw as *mut $t, alloc)
            }))
        })
    };
}

pub type RpcResult = Result<Box<dyn TlObject + Send + Sync>, Box<dyn Error>>;
pub type RpcFunction = for<'a> fn(Arc<Mutex<Session>>, Box<dyn TlObject + Send + Sync + 'a>) -> BoxFuture<'a, RpcResult>;
pub type RpcMapping = Vec<(i32, RpcFunction)>;
