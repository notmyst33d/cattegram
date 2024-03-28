use std::io::{Read, Result, Error, ErrorKind};

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
        self.position = 0;
        self.forward(position);
    }

    pub fn forward(&mut self, length: usize) -> usize {
        if self.position + length > self.data.len() {
            let remainder = self.data.len() - self.position;
            self.position += remainder;
            return remainder;
        }

        self.position += length;
        length
    }

    pub fn backward(&mut self, length: usize) -> usize {
        if self.position.checked_sub(length).is_none() {
            let remainder = self.position;
            self.position -= remainder;
            return remainder;
        }

        self.position -= length;
        length
    }

    pub fn end(&self) -> bool {
        self.position == self.data.len()
    }

    // Convenicence methods
    pub fn read_u8(&mut self) -> Result<u8> {
        let mut buffer = [0u8; 1];
        self.read(&mut buffer)?;
        Ok(u8::from_le_bytes(buffer))
    }

    pub fn read_u16(&mut self) -> Result<u16> {
        let mut buffer = [0u8; 2];
        self.read(&mut buffer)?;
        Ok(u16::from_le_bytes(buffer))
    }

    pub fn read_u32(&mut self) -> Result<u32> {
        let mut buffer = [0u8; 4];
        self.read(&mut buffer)?;
        Ok(u32::from_le_bytes(buffer))
    }

    pub fn read_u128(&mut self) -> Result<u128> {
        let mut buffer = [0u8; 16];
        self.read(&mut buffer)?;
        Ok(u128::from_le_bytes(buffer))
    }

    pub fn read_string(&mut self) -> Result<String> {
        todo!();
    }
}

impl Read for BytesBuffer {
    fn read(&mut self, buffer: &mut [u8]) -> Result<usize> {
        let old_position = self.position;
        let length = buffer.len();

        if let Some(data) = self.data.get(self.position..self.position + length) {
            buffer.clone_from_slice(data);
        } else {
            return Err(Error::new(ErrorKind::Other, "Not enough data"));
        }

        self.position += length;
        Ok(self.position - old_position)
    }
}
