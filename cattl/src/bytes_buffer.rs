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

    // Convenicence methods
    pub fn read_u8(&mut self) -> u8 {
        self.position += 1;
        unsafe {
            (self.raw_ptr.add(self.position - 1)).read_unaligned()
        }
    }

    pub fn read_u32(&mut self) -> u32 {
        self.position += 4;
        unsafe {
            (self.raw_ptr.add(self.position - 4) as *const u32).read_unaligned()
        }
    }

    pub fn read_u64(&mut self) -> u64 {
        self.position += 8;
        unsafe {
            (self.raw_ptr.add(self.position - 8) as *const u64).read_unaligned()
        }
    }

    pub fn read_u128(&mut self) -> u128 {
        self.position += 16;
        unsafe {
            (self.raw_ptr.add(self.position - 16) as *const u128).read_unaligned()
        }
    }

    pub fn read_bytes(&mut self) -> &'static [u8] {
        let length = self.read_u8() as usize;
        self.position += length;
        unsafe {
            std::slice::from_raw_parts(self.raw_ptr.add(self.position - length), length)
        }
    }
}
