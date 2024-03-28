mod tl_object;
mod bytes_buffer;
mod mtproto;

use std::time::{Duration, SystemTime};
use crate::tl_object::TlObject;
use crate::bytes_buffer::BytesBuffer;
use crate::mtproto::*;

fn main() {
    let mut data = BytesBuffer::new(std::fs::read("data/data.bin").unwrap());
    tl_object::add_reader(0xbe7e8ef1, read_req_pq_multi);

    let mut i = 0;
    let duration = Duration::from_secs(1);
    let start = SystemTime::now();
    loop {
        let obj = tl_object::read(&mut data).unwrap();

        // Very stinky cast, but should be safe
        let req_pq_multi = unsafe {
            ((&*obj as *const dyn TlObject) as *const ReqPqMulti).as_ref().unwrap()
        };

        assert!(req_pq_multi.nonce == 123);
        data.seek(0);
        i += 1;
        if start.elapsed().unwrap() >= duration {
            break;
        }
    }

    println!("Decoded {} objects in {} second(s)", i, start.elapsed().unwrap().as_secs());
}
