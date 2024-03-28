use crate::bytes_buffer::BytesBuffer;

static mut READERS: Vec<(u32, fn(&mut BytesBuffer) -> Option<Box<dyn TlObject>>)> = vec![];

pub trait TlObject {
    fn hash(&self) -> u32;
}

pub fn add_reader(hash: u32, reader: fn(&mut BytesBuffer) -> Option<Box<dyn TlObject>>) {
    unsafe { READERS.push((hash, reader)); }
}

pub fn read(data: &mut BytesBuffer) -> Option<Box<dyn TlObject>> {
    let hash = data.read_u32();

    unsafe {
        for reader in &READERS {
            if reader.0 == hash {
                return reader.1(data);
            }
        }
    }

    None
}
