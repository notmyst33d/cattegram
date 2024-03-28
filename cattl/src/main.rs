mod tl_object;
mod bytes_buffer;
mod mtproto;

use std::time::{Duration, SystemTime};
use crate::tl_object::TlObject;
use crate::bytes_buffer::BytesBuffer;
use crate::mtproto::*;

fn main() {
    let mut data = BytesBuffer::new(std::fs::read("data/resPQ.bin").unwrap());
    tl_object::add_reader(0xbe7e8ef1, read_req_pq_multi);
    tl_object::add_reader(0x05162463, read_resPQ);

    let mut i = 0;
    let duration = Duration::from_secs(1);
    let start = SystemTime::now();
    loop {
        let obj = tl_object::read(&mut data).unwrap();

        // Very stinky cast, but should be safe
        match obj.hash() {
            0xbe7e8ef1 => unsafe {
                let obj = ((&*obj as *const dyn TlObject) as *const req_pq_multi).as_ref().unwrap();
                assert!(obj.nonce == 123);
            },
            0x05162463 => unsafe {
                let obj = ((&*obj as *const dyn TlObject) as *const resPQ).as_ref().unwrap();
                if i == 0 {
                    println!("{:#?}", obj);
                }
                assert!(obj.nonce == 123);
                assert!(obj.server_nonce == 123);
            },
            _ => panic!("Oh no, bad cast!"),
        };

        data.seek(0);
        i += 1;
        if start.elapsed().unwrap() >= duration {
            break;
        }
    }

    println!("Decoded {} objects in {} second(s)", i, start.elapsed().unwrap().as_secs());
}
