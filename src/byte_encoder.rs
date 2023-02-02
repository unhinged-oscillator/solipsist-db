use std::io::{self, Read, Write};

pub struct ByteEncoder<T: Write> {
    pub inner: T,
}

pub struct ByteDecoder<T: Read> {
    inner: T,
}

impl<T: Read> ByteDecoder<T> {
    pub fn new(inner: T) -> Self {
        ByteDecoder { inner }
    }

    pub fn read_u8(&mut self) -> io::Result<u8> {
        let mut buf = [0; 1];
        self.inner.read_exact(&mut buf)?;
        Ok(buf[0])
    }

    pub fn read_u16(&mut self) -> io::Result<u16> {
        let mut buf = [0; 2];
        self.inner.read_exact(&mut buf)?;
        let value = u16::from_le_bytes(buf);
        Ok(value)
    }

    pub fn read_u32(&mut self) -> io::Result<u32> {
        let mut buf = [0; 4];
        self.inner.read_exact(&mut buf)?;
        let value = u32::from_le_bytes(buf);
        Ok(value)
    }

    pub fn read_u64(&mut self) -> io::Result<u64> {
        let mut buf = [0; 8];
        self.inner.read_exact(&mut buf)?;
        let value = u64::from_le_bytes(buf);
        Ok(value)
    }

    pub fn read_u128(&mut self) -> io::Result<u128> {
        let mut buf = [0; 16];
        self.inner.read_exact(&mut buf)?;
        let value = u128::from_le_bytes(buf);
        Ok(value)
    }
}

impl<T: Write> ByteEncoder<T> {
    pub fn new(inner: T) -> Self {
        ByteEncoder { inner }
    }

    pub fn write_u8(&mut self, value: u8) -> io::Result<()> {
        let buf = value.to_le_bytes();
        self.inner.write_all(&buf)?;
        Ok(())
    }

    pub fn write_u16(&mut self, value: u16) -> io::Result<()> {
        let buf = value.to_le_bytes();
        self.inner.write_all(&buf)?;
        Ok(())
    }

    pub fn write_u32(&mut self, value: u32) -> io::Result<()> {
        let buf = value.to_le_bytes();
        self.inner.write_all(&buf)?;
        Ok(())
    }

    pub fn write_u64(&mut self, value: u64) -> io::Result<()> {
        let buf = value.to_le_bytes();
        self.inner.write_all(&buf)?;
        Ok(())
    }

    pub fn write_u128(&mut self, value: u128) -> io::Result<()> {
        let buf = value.to_le_bytes();
        self.inner.write_all(&buf)?;
        Ok(())
    }
}
