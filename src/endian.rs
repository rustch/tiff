//! This module handles endianness reading.
use std::io::{Read, Result, Seek, SeekFrom};

/// A simple enum representing known endianness.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Endian {
    Big,
    Little,
}

impl Endian {
    pub fn adjust_u16(&self, x: u16) -> u16 {
        match self {
            Endian::Big => u16::from_be(x),
            Endian::Little => u16::from_le(x),
        }
    }

    pub fn adjust_u32(&self, x: u32) -> u32 {
        match self {
            Endian::Big => u32::from_be(x),
            Endian::Little => u32::from_le(x),
        }
    }

    pub fn adjust_u64(&self, x: u64) -> u64 {
        match self {
            Endian::Big => u64::from_be(x),
            Endian::Little => u64::from_le(x),
        }
    }
}

/// A reader aware of endianness
pub struct EndianReader<'a, R: 'a> {
    inner: &'a mut R,
    endian: Endian,
}

impl<'a, R: Seek> Seek for EndianReader<'a, R> {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        self.inner.seek(pos)
    }
}

impl<'a, R: Read> EndianReader<'a, R> {
    /// Creates an `EndianReader` from a specific reader
    /// and `Endian` value.
    pub fn new(reader: &mut R, endian: Endian) -> EndianReader<R> {
        EndianReader {
            inner: reader,
            endian,
        }
    }

    /// Read one `u16` from the reader.
    pub fn read_u16(&mut self) -> Result<u16> {
        let mut buf: [u8; 2] = [0; 2];
        self.inner.read_exact(&mut buf)?;

        let value = u16::from_bytes(buf);
        Ok(self.endian.adjust_u16(value))
    }

    /// Read one `u32` from the reader.
    pub fn read_u32(&mut self) -> Result<u32> {
        let mut buf: [u8; 4] = [0; 4];
        self.inner.read_exact(&mut buf)?;
        let value = u32::from_bytes(buf);
        let ret = match self.endian {
            Endian::Big => u32::from_be(value),
            Endian::Little => u32::from_le(value),
        };
        Ok(ret)
    }

    /// Read one `u64` from the reader.
    pub fn read_u64(&mut self) -> Result<u64> {
        let mut buf: [u8; 8] = [0; 8];
        self.inner.read_exact(&mut buf)?;

        let value = u64::from_bytes(buf);
        let ret = match self.endian {
            Endian::Big => u64::from_be(value),
            Endian::Little => u64::from_le(value),
        };
        Ok(ret)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    #[test]
    fn test_reader() {
        let bytes: Vec<u8> = vec![
            0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE,
        ];
        let mut cursor = Cursor::new(&bytes);
        {
            let mut be_reader = EndianReader::new(&mut cursor, Endian::Big);
            assert_eq!(0x1122, be_reader.read_u16().unwrap());
            assert_eq!(0x33445566, be_reader.read_u32().unwrap());
            assert_eq!(0x778899AABBCCDDEE, be_reader.read_u64().unwrap());
        }

        cursor.set_position(0);
        {
            let mut le_reader = EndianReader::new(&mut cursor, Endian::Little);
            assert_eq!(0x2211, le_reader.read_u16().unwrap());
            assert_eq!(0x66554433, le_reader.read_u32().unwrap());
            assert_eq!(0xEEDDCCBBAA998877, le_reader.read_u64().unwrap());
        }
    }
}
