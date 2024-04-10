use crate::BytesBuffer;

pub trait TlObject {
    fn hash(&self) -> i32;
    fn write(&self, data: &mut BytesBuffer);
}

