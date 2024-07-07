use std::{fmt, error};
use std::ptr::copy_nonoverlapping;

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
        pub fn $n(&mut self) -> Result<$t, TlBufferError> {
            if self.position + $s > self.data.len() {
                return Err(TlBufferError::NotEnoughData("$t".to_string()));
            }
            self.position += $s;
            Ok(unsafe {
                (self.data.as_ptr().add(self.position - $s) as *const $t).read_unaligned()
            })
        }
    };
}

#[derive(Debug)]
pub struct TlBuffer {
    data: Vec<u8>,
    position: usize,
}

impl TlBuffer {
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

    pub fn len(&self) -> usize {
        self.data.len()
    }

    impl_primitive_read!(read_byte, 1, i8);
    impl_primitive_read!(read_int, 4, i32);
    impl_primitive_read!(read_long, 8, i64);
    impl_primitive_read!(read_double, 8, f64);
    impl_primitive_read!(read_int128, 16, i128);

    impl_primitive_write!(write_byte, 1, i8);
    impl_primitive_write!(write_int, 4, i32);
    impl_primitive_write!(write_long, 8, i64);
    impl_primitive_write!(write_double, 8, f64);
    impl_primitive_write!(write_int128, 16, i128);

    #[inline]
    pub fn read_int24(&mut self) -> Result<u32, TlBufferError> {
        if self.position + 3 > self.data.len() {
            return Err(TlBufferError::NotEnoughData("int24".to_string()));
        }
        self.position += 3;
        Ok(unsafe {
            (self.data.as_ptr().add(self.position - 3) as *const u8).read_unaligned() as u32 |
            ((self.data.as_ptr().add(self.position - 2) as *const u8).read_unaligned() as u32) << 8 |
            ((self.data.as_ptr().add(self.position - 1) as *const u8).read_unaligned() as u32) << 16
        })
    }

    #[inline]
    pub fn write_int24(&mut self, value: i32) {
        if self.position + 3 > self.data.len() {
            self.data.resize(self.data.len() + 3, 0);
        }
        self.position += 3;
        let b1 = value & 0xff;
        let b2 = (value >> 8) & 0xff;
        let b3 = (value >> 16) & 0xff;
        unsafe {
            (self.data.as_ptr().add(self.position - 3) as *mut i8).write_unaligned(b1 as i8);
            (self.data.as_ptr().add(self.position - 2) as *mut i8).write_unaligned(b2 as i8);
            (self.data.as_ptr().add(self.position - 1) as *mut i8).write_unaligned(b3 as i8);
        }
    }

    pub fn read_bytes(&mut self) -> Result<Vec<u8>, TlBufferError> {
        let b = self.read_byte()? as u8;
        let mut additional_length = 1;
        let length = if b >= 0xfe {
            additional_length = 0;
            usize::try_from(self.read_int24()?).unwrap()
        } else {
            (b as usize) & 0xff
        };
        if self.position + length > self.data.len() {
            return Err(TlBufferError::NotEnoughData("bytes".to_string()));
        }
        let padding = (4 - ((length + additional_length) % 4)) % 4;
        self.position += length + padding;
        Ok(self.data[self.position - (length + padding)..self.position - padding].to_vec())
    }

    pub fn write_bytes(&mut self, value: &[u8]) {
        let mut additional_length = 1;
        if value.len() >= 0xfe {
            if self.position + value.len() + 4 > self.data.len() {
                self.data.resize(self.data.len() + value.len() + 4, 0);
            }
            self.write_byte(0xfeu8 as i8);
            self.write_int24(value.len() as i32);
            additional_length = 0;
        } else {
            if self.position + value.len() + 1 > self.data.len() {
                self.data.resize(self.data.len() + value.len() + 1, 0);
            }
            self.write_byte(value.len() as i8);
        }
        self.position += value.len();
        unsafe {
            copy_nonoverlapping(
                value.as_ptr(),
                self.data.as_ptr().add(self.position - value.len()) as *mut u8,
                value.len(),
            )
        }
        let padding = (4 - ((value.len() + additional_length) % 4)) % 4;
        for _ in 0..padding {
            self.write_byte(0);
        }
    }

    pub fn read_string(&mut self) -> Result<String, TlBufferError> {
        match String::from_utf8(self.read_bytes()?) {
            Ok(result) => Ok(result),
            Err(_) => Err(TlBufferError::BadData),
        }
    }

    pub fn write_string(&mut self, value: &String) {
        self.write_bytes(value.as_bytes())
    }

    pub fn read_bool(&mut self) -> Result<bool, TlBufferError> {
        match self.read_int()? {
            -1720552011 | 1072550713 => Ok(true),
            -1132882121 => Ok(false),
            _ => Err(TlBufferError::BadData),
        }
    }

    pub fn write_bool(&mut self, value: bool) {
        self.write_int(if value { -1720552011 } else { -1132882121 })
    }

    pub fn read_raw(&mut self, length: usize) -> Result<Vec<u8>, TlBufferError> {
        if self.position + length > self.data.len() {
            return Err(TlBufferError::NotEnoughData("raw".to_string()));
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
            copy_nonoverlapping(
                value.as_ptr(),
                self.data.as_ptr().add(self.position - value.len()) as *mut u8,
                value.len(),
            )
        }
    }
}

impl Into<TlBuffer> for Vec<u8> {
    fn into(self) -> TlBuffer {
        TlBuffer::new(self)
    }
}

impl Into<TlBuffer> for &[u8] {
    fn into(self) -> TlBuffer {
        TlBuffer::new(self.to_vec())
    }
}

#[derive(Debug, Clone)]
pub enum TlBufferError {
    NotEnoughData(String),
    BadData,
    Custom(String),
}

impl fmt::Display for TlBufferError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TlBufferError::NotEnoughData(source) => write!(f, "not enough data when reading {}", source),
            TlBufferError::BadData => write!(f, "bad data"),
            TlBufferError::Custom(message) => write!(f, "{}", message),
        }
    }
}

impl error::Error for TlBufferError {}
