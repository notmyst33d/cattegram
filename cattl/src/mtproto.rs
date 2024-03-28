use crate::tl_object::TlObject;
use crate::bytes_buffer::BytesBuffer;

#[derive(Default)]
pub struct ReqPqMulti {
    pub hash: u32,
    pub nonce: u128,
}

impl TlObject for ReqPqMulti {
    fn hash(&self) -> u32 {
        self.hash
    }
}

pub fn read_req_pq_multi(data: &mut BytesBuffer) -> Option<Box<dyn TlObject>> {
    let mut obj = ReqPqMulti::default();

    obj.hash = 0xbe7e8ef1;
    obj.nonce = data.read_u128();

    Some(Box::new(obj))
}
