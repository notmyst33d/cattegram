mod prime;
mod rpc;
mod rsa;
mod session;
mod storage;
mod tcp_abridged_combined;
mod transport;

use crate::session::Session;
use aes::cipher::{KeyIvInit, StreamCipher};
use catte_tl_schema::{RpcError, RpcResult, SchemaObject};
use std::error::Error;
use std::sync::Arc;
use tcp_abridged_combined::TcpAbridgedCombined;
use tokio::io::AsyncReadExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::runtime::Builder;
use tokio::sync::Mutex;
use transport::Transport;

type Aes256Ctr = ctr::Ctr32BE<aes::Aes256>;

#[macro_export]
macro_rules! clone_sized_slice {
    ($v:expr, $s:expr) => {{
        let mut s = [0u8; $s];
        s[..$v.len()].clone_from_slice($v);
        s
    }};
}

#[macro_export]
macro_rules! hex_string {
    ($v:expr) => {
        $v.iter().map(|v| format!("{:02x}", v)).collect::<String>()
    };
}

#[macro_export]
macro_rules! println_yellow {
    ($t:literal, $($arg:tt)*) => {
        println!("\x1b[43;37;1m {} \x1b[0m {}", $t, format!($($arg)*))
    };
}

#[macro_export]
macro_rules! println_red {
    ($t:literal, $($arg:tt)*) => {
        println!("\x1b[41;37;1m {} \x1b[0m {}", $t, format!($($arg)*))
    };
}

#[macro_export]
macro_rules! println_blue {
    ($t:literal, $($arg:tt)*) => {
        println!("\x1b[44;37;1m {} \x1b[0m {}", $t, format!($($arg)*))
    };
}

#[macro_export]
macro_rules! time {
    () => {{
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i32
    }};
}

async fn client_thread(mut socket: TcpStream) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut buf = [0u8; 1];

    socket.read_exact(&mut buf[..1]).await?;

    let transport: Box<dyn Transport> = if buf[0] == 0xef {
        Box::new(TcpAbridgedCombined::new(socket, None, None))
    } else {
        let mut nonce = [0u8; 64];
        nonce[0] = buf[0];
        socket.read_exact(&mut nonce[1..]).await?;
        let nonce_reversed = nonce[8..56].iter().cloned().rev().collect::<Vec<u8>>();
        let encrypt_key = clone_sized_slice!(&nonce[8..40], 32);
        let encrypt_iv = clone_sized_slice!(&nonce[40..56], 16);
        let decrypt_key = clone_sized_slice!(&nonce_reversed[..32], 32);
        let decrypt_iv = clone_sized_slice!(&nonce_reversed[32..48], 16);
        let mut encrypt = Aes256Ctr::new(&encrypt_key.into(), &encrypt_iv.into());
        let decrypt = Aes256Ctr::new(&decrypt_key.into(), &decrypt_iv.into());
        encrypt.apply_keystream(&mut nonce);
        Box::new(TcpAbridgedCombined::new(
            socket,
            Some(encrypt),
            Some(decrypt),
        ))
    };

    let session = Arc::new(Mutex::new(Session::new(transport)));

    loop {
        let messages = session.lock().await.receive().await?;
        let mut responses = vec![];
        for message in messages {
            match message.2 {
                SchemaObject::DeserializationError(e) => {
                    println_yellow!("TL ERROR", "{}", e);
                    responses.push(SchemaObject::RpcResult(RpcResult {
                        req_msg_id: message.0,
                        result: Box::new(SchemaObject::RpcError(RpcError {
                            error_code: 0,
                            error_message: format!("deserialization error: {}", e),
                        })),
                    }));
                }
                SchemaObject::MsgsAck(_) => continue,
                SchemaObject::RpcResult(_) => continue,
                _ => {
                    println_red!("REQUEST", "{:?}", message.2);
                    let response = match rpc::invoke(session.clone(), message).await {
                        Ok(result) => result,
                        Err(e) => {
                            println_yellow!("RPC ERROR", "{}", e);
                            SchemaObject::RpcError(RpcError {
                                error_code: 0,
                                error_message: e.to_string(),
                            })
                        }
                    };
                    println_blue!("RESPONSE", "{:?}", response);
                    responses.push(response);
                }
            }
        }
        if responses.len() != 0 {
            session.lock().await.send(responses).await?;
        }
    }
}

async fn async_main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("0.0.0.0:443").await?;
    println!("Telecat started");

    loop {
        let (socket, _) = listener.accept().await?;
        tokio::spawn(async {
            match client_thread(socket).await {
                Ok(_) => {}
                Err(e) => println!("client returned an error: {}", e),
            }
        });
    }
}

fn main() {
    Builder::new_multi_thread()
        .enable_all()
        .thread_stack_size(10 * 1024 * 1024)
        .build()
        .unwrap()
        .block_on(async_main())
        .unwrap();
}
