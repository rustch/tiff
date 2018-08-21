//! This module handles endianness reading.
use std::io::{Read, Result, Seek, SeekFrom};

/// A simple enum representing known endianness.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Endian {
    Big,
    Little,
}

/// A constant representing a Big endianness;
pub const BE: Endian = Endian::Big;
/// A constant representing a Little endianness;
pub const LE: Endian = Endian::Little;

pub trait Byte: Copy + Sized {
    fn from_bytes_le(bytes: [u8; 1]) -> Self;
    fn from_bytes_be(bytes: [u8; 1]) -> Self;
    fn to_bytes_le(self) -> [u8; 1];
    fn to_bytes_be(self) -> [u8; 1];
}

impl Byte for i8 {
    fn from_bytes_le(bytes: [u8; 1]) -> i8 {
        i8::from_le_bytes(bytes)
    }
    fn from_bytes_be(bytes: [u8; 1]) -> i8 {
        i8::from_be_bytes(bytes)
    }
    fn to_bytes_le(self) -> [u8; 1] {
        self.to_le_bytes()
    }
    fn to_bytes_be(self) -> [u8; 1] {
        self.to_be_bytes()
    }
}

impl Byte for u8 {
    fn from_bytes_le(bytes: [u8; 1]) -> u8 {
        u8::from_le_bytes(bytes)
    }
    fn from_bytes_be(bytes: [u8; 1]) -> u8 {
        u8::from_be_bytes(bytes)
    }
    fn to_bytes_le(self) -> [u8; 1] {
        self.to_le_bytes()
    }
    fn to_bytes_be(self) -> [u8; 1] {
        self.to_be_bytes()
    }
}

pub trait Short: Copy + Sized {
    fn from_bytes_le(bytes: [u8; 2]) -> Self;
    fn from_bytes_be(bytes: [u8; 2]) -> Self;
    fn to_bytes_le(self) -> [u8; 2];
    fn to_bytes_be(self) -> [u8; 2];
}

impl Short for u16 {
    fn from_bytes_le(bytes: [u8; 2]) -> u16 {
        u16::from_le_bytes(bytes)
    }
    fn from_bytes_be(bytes: [u8; 2]) -> u16 {
        u16::from_be_bytes(bytes)
    }
    fn to_bytes_le(self) -> [u8; 2] {
        self.to_le_bytes()
    }
    fn to_bytes_be(self) -> [u8; 2] {
        self.to_be_bytes()
    }
}

impl Short for i16 {
    fn from_bytes_le(bytes: [u8; 2]) -> i16 {
        i16::from_le_bytes(bytes)
    }
    fn from_bytes_be(bytes: [u8; 2]) -> i16 {
        i16::from_be_bytes(bytes)
    }

    fn to_bytes_le(self) -> [u8; 2] {
        self.to_le_bytes()
    }
    fn to_bytes_be(self) -> [u8; 2] {
        self.to_be_bytes()
    }
}
pub trait Long: Copy + Sized {
    fn from_bytes_le(bytes: [u8; 4]) -> Self;
    fn from_bytes_be(bytes: [u8; 4]) -> Self;
    fn to_bytes_le(self) -> [u8; 4];
    fn to_bytes_be(self) -> [u8; 4];
}

impl Long for u32 {
    fn from_bytes_le(bytes: [u8; 4]) -> u32 {
        u32::from_le_bytes(bytes)
    }
    fn from_bytes_be(bytes: [u8; 4]) -> u32 {
        u32::from_be_bytes(bytes)
    }

    fn to_bytes_le(self) -> [u8; 4] {
        self.to_le_bytes()
    }
    fn to_bytes_be(self) -> [u8; 4] {
        self.to_be_bytes()
    }
}

impl Long for i32 {
    fn from_bytes_le(bytes: [u8; 4]) -> i32 {
        i32::from_le_bytes(bytes)
    }
    fn from_bytes_be(bytes: [u8; 4]) -> i32 {
        i32::from_be_bytes(bytes)
    }
    fn to_bytes_le(self) -> [u8; 4] {
        self.to_le_bytes()
    }
    fn to_bytes_be(self) -> [u8; 4] {
        self.to_be_bytes()
    }
}

pub trait LongLong: Copy + Sized {
    fn from_bytes_le(bytes: [u8; 8]) -> Self;
    fn from_bytes_be(bytes: [u8; 8]) -> Self;

    fn to_bytes_le(self) -> [u8; 8];
    fn to_bytes_be(self) -> [u8; 8];
}

impl LongLong for u64 {
    fn from_bytes_le(bytes: [u8; 8]) -> u64 {
        u64::from_le_bytes(bytes)
    }
    fn from_bytes_be(bytes: [u8; 8]) -> u64 {
        u64::from_be_bytes(bytes)
    }

    fn to_bytes_le(self) -> [u8; 8] {
        self.to_le_bytes()
    }
    fn to_bytes_be(self) -> [u8; 8] {
        self.to_be_bytes()
    }
}

impl LongLong for i64 {
    fn from_bytes_le(bytes: [u8; 8]) -> i64 {
        i64::from_le_bytes(bytes)
    }
    fn from_bytes_be(bytes: [u8; 8]) -> i64 {
        i64::from_be_bytes(bytes)
    }
    fn to_bytes_le(self) -> [u8; 8] {
        self.to_le_bytes()
    }
    fn to_bytes_be(self) -> [u8; 8] {
        self.to_be_bytes()
    }
}

impl Endian {
    pub fn byte_adjusted<T: Byte>(self, val: T) -> [u8; 1] {
        match self {
            Endian::Big => val.to_bytes_be(),
            Endian::Little => val.to_bytes_le(),
        }
    }

    pub fn short_from_bytes<T: Short>(self, bytes: [u8; 2]) -> T {
        match self {
            Endian::Big => T::from_bytes_be(bytes),
            Endian::Little => T::from_bytes_le(bytes),
        }
    }

    pub fn short_adjusted<T: Short>(self, val: T) -> [u8; 2] {
        match self {
            Endian::Big => val.to_bytes_be(),
            Endian::Little => val.to_bytes_le(),
        }
    }

    pub fn long_from_bytes<T: Long>(self, bytes: [u8; 4]) -> T {
        match self {
            Endian::Big => T::from_bytes_be(bytes),
            Endian::Little => T::from_bytes_le(bytes),
        }
    }

    pub fn long_adjusted<T: Long>(self, val: T) -> [u8; 4] {
        match self {
            Endian::Big => val.to_bytes_be(),
            Endian::Little => val.to_bytes_le(),
        }
    }

    pub fn longlong_from_bytes<T: LongLong>(self, bytes: [u8; 8]) -> T {
        match self {
            Endian::Big => T::from_bytes_be(bytes),
            Endian::Little => T::from_bytes_le(bytes),
        }
    }
    pub fn longlong_adjusted<T: LongLong>(self, val: T) -> [u8; 8] {
        match self {
            Endian::Big => val.to_bytes_be(),
            Endian::Little => val.to_bytes_le(),
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

    /// Read short from the reader.
    pub fn read_short<T: Short>(&mut self) -> Result<T> {
        let mut buf: [u8; 2] = [0; 2];
        self.inner.read_exact(&mut buf)?;
        Ok(self.endian.short_from_bytes(buf))
    }

    /// Read long from the reader.
    pub fn read_long<T: Long>(&mut self) -> Result<T> {
        let mut buf: [u8; 4] = [0; 4];
        self.inner.read_exact(&mut buf)?;
        Ok(self.endian.long_from_bytes(buf))
    }

    /// Read long from the reader.
    #[allow(dead_code)]
    pub fn read_longlong<T: LongLong>(&mut self) -> Result<T> {
        let mut buf: [u8; 8] = [0; 8];
        self.inner.read_exact(&mut buf)?;
        Ok(self.endian.longlong_from_bytes(buf))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    #[test]
    fn test_reader() {
        let bytes: Vec<u8> = vec![0x11, 0x22, 0x33, 0x44, 0x55, 0x66];
        let mut cursor = Cursor::new(&bytes);
        {
            let mut be_reader = EndianReader::new(&mut cursor, Endian::Big);
            assert_eq!(0x1122u16, be_reader.read_short().unwrap());
            assert_eq!(0x33445566u32, be_reader.read_long().unwrap());
        }

        cursor.set_position(0);
        {
            let mut le_reader = EndianReader::new(&mut cursor, Endian::Little);
            assert_eq!(0x2211u16, le_reader.read_short().unwrap());
            assert_eq!(0x66554433u32, le_reader.read_long().unwrap());
        }
    }
}
