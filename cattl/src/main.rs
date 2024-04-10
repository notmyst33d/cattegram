use std::time::{Duration, SystemTime};
use cattl::{TlObject, TlReader, BytesBuffer, mtproto};

fn main() {
    let mut data = BytesBuffer::new(std::fs::read("data/req_pq_multi.bin").unwrap());
    let reader = TlReader::new();

    let mut i = 0;
    let duration = Duration::from_secs(1);
    let start = SystemTime::now();
    loop {
        let obj = reader.read(&mut data).unwrap();
        let rpq = unsafe { (&*obj as *const dyn TlObject as *const mtproto::req_pq_multi).as_ref().unwrap() };
        assert!(rpq.nonce == 123);

        data.seek(0);
        i += 1;
        if start.elapsed().unwrap() >= duration {
            break;
        }
    }

    println!("Decoded {} objects in {} second(s)", i, start.elapsed().unwrap().as_secs());
}
