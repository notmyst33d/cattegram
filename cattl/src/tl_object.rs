use std::any::Any;
use crate::BytesBuffer;

pub trait TlObject: Any {
    fn hash(&self) -> i32;
    fn write(&self, data: &mut BytesBuffer);
}

