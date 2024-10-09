pub mod account;
pub mod auth;
pub mod base;
pub mod contacts;
pub mod help;
pub mod langpack;
pub mod messages;
pub mod mtproto;
pub mod updates;
pub mod users;

pub use account::*;
pub use auth::*;
pub use base::*;
pub use contacts::*;
pub use help::*;
pub use langpack::*;
pub use messages::*;
pub use mtproto::*;
pub use updates::*;
pub use users::*;

#[macro_export]
macro_rules! ok {
    ($message:tt, $objtype:tt $body:tt) => {
        return Ok(SchemaObject::RpcResult(RpcResult {
            req_msg_id: $message.msg_id,
            result: Box::new(SchemaObject::$objtype($objtype $body)),
        }))
    };
}

#[macro_export]
macro_rules! ok_user {
    ($message:tt, $user:expr) => {
        return Ok(SchemaObject::RpcResult(RpcResult {
            req_msg_id: $message.msg_id,
            result: Box::new(SchemaObject::User($user)),
        }))
    };
}

#[macro_export]
macro_rules! err {
    ($message:tt, $code:expr, $emessage:expr) => {
        return Ok(SchemaObject::RpcResult(RpcResult {
            req_msg_id: $message.msg_id,
            result: Box::new(SchemaObject::RpcError(RpcError {
                error_code: $code,
                error_message: $emessage.to_string(),
            })),
        }))
    };
}

#[macro_export]
macro_rules! ok_raw {
    ($objtype:tt $body:tt) => {
        Ok(SchemaObject::$objtype($objtype $body))
    };
}

#[macro_export]
macro_rules! ok_vec {
    ($message:tt, $body:expr) => {
        Ok(SchemaObject::RpcResult(RpcResult {
            req_msg_id: $message.msg_id,
            result: Box::new(SchemaObject::Vector($body)),
        }))
    };
}

#[macro_export]
macro_rules! v {
    ($vtype:tt::$objtype:tt $body:tt) => {
        $vtype::$objtype(Box::new($objtype $body))
    };
}

include!(concat!(env!("OUT_DIR"), "/generated_invoke.rs"));
