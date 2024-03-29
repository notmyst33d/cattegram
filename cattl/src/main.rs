mod tl_object;
mod bytes_buffer;
mod mtproto;

use std::time::{Duration, SystemTime};
use crate::bytes_buffer::BytesBuffer;
use crate::mtproto::*;

fn main() {
    let mut data = BytesBuffer::new(std::fs::read("data/req_pq_multi.bin").unwrap());
    init();

    let mut i = 0;
    let duration = Duration::from_secs(1);
    let start = SystemTime::now();
    loop {
        let obj = tl_object::read(&mut data).unwrap();
        let rpq = obj.downcast::<req_pq_multi>().unwrap();
        assert!(rpq.nonce == 123);

        data.seek(0);
        i += 1;
        if start.elapsed().unwrap() >= duration {
            break;
        }
    }

    println!("Decoded {} objects in {} second(s)", i, start.elapsed().unwrap().as_secs());
}
