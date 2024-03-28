use core::any::Any;

use crate::bytes_buffer::BytesBuffer;

#[derive(Default)]
pub struct ReqPqMulti {
    pub hash: u32,
    pub nonce: u128,
}

pub fn read_req_pq_multi(data: &mut BytesBuffer) -> Option<Box<dyn Any>> {
    let mut obj = ReqPqMulti::default();

    obj.hash = 0xbe7e8ef1;
    if let Ok(nonce) = data.read_u128() {
        obj.nonce = nonce;
    } else {
        return None;
    }

    Some(Box::new(obj))
}
