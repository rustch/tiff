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

pub trait EndianType: Sized + Clone + Copy {
    fn from_be(x: Self) -> Self;
    fn from_le(x: Self) -> Self;
}

pub trait Short: EndianType {
    fn from_bytes(bytes: [u8; 2]) -> Self;
}

pub trait Long: EndianType {
    fn from_bytes(bytes: [u8; 4]) -> Self;
}

pub trait LongLong: EndianType {
    fn from_bytes(bytes: [u8; 8]) -> Self;
}

macro_rules! EndianTypeImpl {
    ($t:ident) => {
        impl EndianType for $t {
            fn from_be(x: $t) -> $t {
                $t::from_be(x)
            }

            fn from_le(x: $t) -> $t {
                $t::from_le(x)
            }
        }
    };
}
macro_rules! ShortImpl {
    ($t:ident) => {
        EndianTypeImpl!($t);

        impl Short for $t {
            fn from_bytes(bytes: [u8; 2]) -> Self {
                $t::from_bytes(bytes)
            }
        }
    };
}

macro_rules! LongImpl {
    ($t:ident) => {
        EndianTypeImpl!($t);

        impl Long for $t {
            fn from_bytes(bytes: [u8; 4]) -> Self {
                $t::from_bytes(bytes)
            }
        }
    };
}

macro_rules! LongLongImpl {
    ($t:ident) => {
        EndianTypeImpl!($t);

        impl LongLong for $t {
            fn from_bytes(bytes: [u8; 8]) -> Self {
                $t::from_bytes(bytes)
            }
        }
    };
}

ShortImpl!(u16);
LongImpl!(u32);
LongLongImpl!(u64);

ShortImpl!(i16);
LongImpl!(i32);
LongLongImpl!(i64);

impl Endian {
    pub fn adjust<T: EndianType>(&self, x: T) -> T {
        match self {
            Endian::Big => T::from_be(x),
            Endian::Little => T::from_le(x),
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
        Ok(self.endian.adjust(value))
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
            assert_eq!(0x1122, be_reader.read_u16().unwrap());
            assert_eq!(0x33445566, be_reader.read_u32().unwrap());
        }

        cursor.set_position(0);
        {
            let mut le_reader = EndianReader::new(&mut cursor, Endian::Little);
            assert_eq!(0x2211, le_reader.read_u16().unwrap());
            assert_eq!(0x66554433, le_reader.read_u32().unwrap());
        }
    }
}
