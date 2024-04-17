use std::ptr::copy_nonoverlapping;
use crate::TlError;

macro_rules! impl_primitive_write {
    ($n:tt, $s:expr, $t:ty) => {
        #[inline]
        pub fn $n(&mut self, value: $t) {
            if self.position + $s > self.data.len() {
                self.data.resize(self.data.len() + $s, 0);
            }
            self.position += $s;
            unsafe {
                (self.data.as_ptr().add(self.position - $s) as *mut $t).write_unaligned(value)
            }
        }
    };
}

macro_rules! impl_primitive_read {
    ($n:tt, $s:expr, $t:ty) => {
        #[inline]
        pub fn $n(&mut self) -> Result<$t, TlError> {
            if self.position + $s > self.data.len() {
                return Err(TlError::NotEnoughData);
            }
            self.position += $s;
            Ok(unsafe {
                (self.data.as_ptr().add(self.position - $s) as *const $t).read_unaligned()
            })
        }
    };
}

#[derive(Debug)]
pub struct BytesBuffer {
    data: Vec<u8>,
    position: usize,
}

impl BytesBuffer {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data, position: 0 }
    }

    pub fn position(&self) -> usize {
        self.position
    }

    pub fn seek(&mut self, position: usize) {
        self.position = position;
    }

    pub fn data(&self) -> &[u8] {
        self.data.as_slice()
    }

    impl_primitive_read!(read_byte, 1, i8);
    impl_primitive_read!(read_int, 4, i32);
    impl_primitive_read!(read_long, 8, i64);
    impl_primitive_read!(read_int128, 16, i128);

    impl_primitive_write!(write_byte, 1, i8);
    impl_primitive_write!(write_int, 4, i32);
    impl_primitive_write!(write_long, 8, i64);
    impl_primitive_write!(write_int128, 16, i128);

    // Thanks Durov, very cool
    #[inline]
    pub fn read_int24(&mut self) -> Result<i32, TlError> {
        if self.position + 3 > self.data.len() {
            return Err(TlError::NotEnoughData);
        }
        self.position += 3;
        let mut value = 0;
        unsafe {
            value |= (self.data.as_ptr().add(self.position - 3) as *const i8).read_unaligned() as i32;
            value |= ((self.data.as_ptr().add(self.position - 2) as *const i8).read_unaligned() as i32) << 8;
            value |= ((self.data.as_ptr().add(self.position - 1) as *const i8).read_unaligned() as i32) << 16;
        }
        Ok(value)
    }

    pub fn read_bytes(&mut self) -> Result<Vec<u8>, TlError> {
        if self.position + 1 > self.data.len() {
            return Err(TlError::NotEnoughData);
        }
        let mut length = self.read_byte()? as usize;
        if (length & 0xff) == 0xfe {
            length = self.read_int24()? as usize;
        }
        if self.position + length > self.data.len() {
            return Err(TlError::NotEnoughData);
        }
        let padding = 4 - (length + 1).rem_euclid(4);
        self.position += length + padding;
        Ok(self.data[self.position - (length + padding)..self.position - padding].to_vec())
    }

    pub fn write_bytes(&mut self, value: &[u8]) {
        if self.position + value.len() + 1 > self.data.len() {
            self.data.resize(self.data.len() + value.len() + 1, 0);
        }
        self.write_byte(value.len() as i8);
        self.position += value.len();
        unsafe {
            copy_nonoverlapping(value.as_ptr(), self.data.as_ptr().add(self.position - value.len()) as *mut u8, value.len())
        }
        let padding = 4 - (value.len() + 1).rem_euclid(4);
        for _ in 0..padding {
            self.write_byte(0);
        }
    }

    pub fn read_raw(&mut self, length: usize) -> Result<Vec<u8>, TlError> {
        if self.position + length > self.data.len() {
            return Err(TlError::NotEnoughData);
        }
        self.position += length;
        Ok(self.data[self.position - length..self.position].to_vec())
    }

    pub fn write_raw(&mut self, value: &[u8]) {
        if self.position + value.len() > self.data.len() {
            self.data.resize(self.data.len() + value.len(), 0);
        }
        self.position += value.len();
        unsafe {
            copy_nonoverlapping(value.as_ptr(), self.data.as_ptr().add(self.position - value.len()) as *mut u8, value.len())
        }
    }
}
