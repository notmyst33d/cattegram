use std::error::Error;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use cattl::BytesBuffer;
use cattl::tl_object::TlObject;
use cattl::mtproto::*;

async fn read_abridged(socket: &mut TcpStream) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut buf = vec![0];
    socket.read_exact(&mut buf[..1]).await?;
    let mut length = buf[0] as usize;
    if length == 0x7f {
        todo!();
    } else {
        length = length * 4;
    }
    buf.resize(length, 0);
    socket.read_exact(&mut buf[..length]).await?;
    Ok(buf)
}

async fn write_abridged(socket: &mut TcpStream, data: Vec<u8>) -> Result<(), Box<dyn Error>> {
    socket.write(&[(data.len() / 4) as u8]).await?;
    socket.write(&data).await?;
    Ok(())
}

fn pack_mtproto(auth_key_id: i64, msg_id: i64, data: Vec<u8>) -> Vec<u8> {
    let mut buffer = BytesBuffer::new(vec![]);
    buffer.write_long(auth_key_id);
    buffer.write_long(msg_id);
    buffer.write_int(data.len() as i32);
    buffer.write_raw(&data);
    buffer.data().to_vec()
}

fn unpack_mtproto(message_data: Vec<u8>) -> Option<(i64, i64, Vec<u8>)> {
    let mut buffer = BytesBuffer::new(message_data);
    let auth_key_id = buffer.read_long()?;
    let msg_id = buffer.read_long()?;
    let length = buffer.read_int()? as usize;
    Some((auth_key_id, msg_id, buffer.read_raw(length)?.to_vec()))
}

async fn read_raw<T>(socket: &mut TcpStream) -> Result<T, Box<dyn Error>> {
    let mut data = BytesBuffer::new(pack_mtproto(0, 0, read_abridged(&mut socket).await?));
    cattl::read(&mut data).unwrap().downcast::<T>()
}

fn write_raw<T: TlObject>(socket: &mut TcpStream, obj: Box<dyn TlObject>) -> Result<(), Box<dyn Error>> {
    Ok(())
}

async fn client_thread(mut socket: TcpStream) -> Result<(), Box<dyn Error>> {
    let mut buf = [0u8; 1];

    socket.read_exact(&mut buf[..1]).await?;

    if buf[0] != 0xef {
        println!("Unsupported transport, only TCP Abridged is supported currently");
        socket.shutdown().await?;
        return Ok(());
    }

    let req = read_raw::<req_pq_multi>(&mut socket).await?;

    /*let res = resPQ {
        nonce: obj.nonce,
        server_nonce: 123,
        pq: b"AAAAAAAA",
        server_public_key_fingerprints: vec![123, 321],
    };*/

    //write_raw(&mut socket, res).await()?;
    //let mut resbuf = BytesBuffer::new(vec![]);
    //res.write(&mut resbuf);
    //let response = pack_mtproto(0, 0, resbuf.data());

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    cattl::mtproto::init();

    let listener = TcpListener::bind("127.0.0.1:8443").await?;
    println!("TCP Server started");

    loop {
        let (mut socket, _) = listener.accept().await?;
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
