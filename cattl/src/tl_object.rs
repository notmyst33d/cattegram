use core::any::Any;

use crate::bytes_buffer::BytesBuffer;

static mut READERS: Vec<(i32, fn(&mut BytesBuffer) -> Option<Box<dyn Any>>)> = vec![];

pub trait TlObject {
    fn hash(&self) -> i32;
}

pub fn add_reader(hash: i32, reader: fn(&mut BytesBuffer) -> Option<Box<dyn Any>>) {
    unsafe { READERS.push((hash, reader)); }
}

pub fn read(data: &mut BytesBuffer) -> Option<Box<dyn Any>> {
    let hash = data.read_int()?;

    unsafe {
        for reader in &READERS {
            if reader.0 == hash {
                return reader.1(data);
            }
        }
    }

    None
}
