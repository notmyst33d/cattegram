mod tcp_abridged;
mod transport;
mod session;

use std::error::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use cattl::mtproto::*;
use crate::session::Session;
use crate::tcp_abridged::TcpAbridged;

async fn client_thread(mut socket: TcpStream) -> Result<(), Box<dyn Error>> {
    let mut buf = [0u8; 1];

    socket.read_exact(&mut buf[..1]).await?;

    if buf[0] != 0xef {
        println!("Unsupported transport, only TCP Abridged is supported currently");
        socket.shutdown().await?;
        return Ok(());
    }

    let mut session = Session::new(Box::new(TcpAbridged::new(socket)));
    let obj = session.receive_raw::<req_pq_multi>().await?;
    println!("nonce: {}", obj.nonce);

    let res = resPQ {
        nonce: obj.nonce,
        server_nonce: 123,
        pq: b"AAAAAAAA",
        server_public_key_fingerprints: vec![123, 321],
    };

    session.send_raw(&res).await?;
    session.close().await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    cattl::mtproto::init();

    let listener = TcpListener::bind("127.0.0.1:8443").await?;
    println!("TCP Server started");

    loop {
        let (socket, _) = listener.accept().await?;
        tokio::spawn(async move {
            match client_thread(socket).await {
                Ok(_) => {},
                Err(_) => {
                    println!("Client returned an error");
                },
            }
        });
    }
}
