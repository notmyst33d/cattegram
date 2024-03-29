#[derive(Debug)]
pub struct BytesBuffer {
    data: Vec<u8>,
    raw_ptr: *const u8,
    position: usize,
}

impl BytesBuffer {
    pub fn new(data: Vec<u8>) -> Self {
        let ptr = data.as_ptr();
        Self { data, raw_ptr: ptr, position: 0 }
    }

    pub fn position(&self) -> usize {
        self.position
    }

    pub fn seek(&mut self, position: usize) {
        self.position = position;
    }

    pub fn read_byte(&mut self) -> Option<i8> {
        if self.position + 1 > self.data.len() {
            return None;
        }
        self.position += 1;
        Some(unsafe {
            (self.raw_ptr.add(self.position - 1) as *const i8).read_unaligned()
        })
    }

    pub fn read_int(&mut self) -> Option<i32> {
        if self.position + 4 > self.data.len() {
            return None;
        }
        self.position += 4;
        Some(unsafe {
            (self.raw_ptr.add(self.position - 4) as *const i32).read_unaligned()
        })
    }

    pub fn read_long(&mut self) -> Option<i64> {
        if self.position + 8 > self.data.len() {
            return None;
        }
        self.position += 8;
        Some(unsafe {
            (self.raw_ptr.add(self.position - 8) as *const i64).read_unaligned()
        })
    }

    pub fn read_int128(&mut self) -> Option<i128> {
        if self.position + 16 > self.data.len() {
            return None;
        }
        self.position += 16;
        Some(unsafe {
            (self.raw_ptr.add(self.position - 16) as *const i128).read_unaligned()
        })
    }

    pub fn read_bytes(&mut self) -> Option<&'static [u8]> {
        if self.position + 1 > self.data.len() {
            return None;
        }
        let length = self.read_byte()? as usize;
        if self.position + length > self.data.len() {
            return None;
        }
        self.position += length;
        Some(unsafe {
            std::slice::from_raw_parts(self.raw_ptr.add(self.position - length), length)
        })
    }
}
