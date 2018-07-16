use std::io::{Read, Result, Seek, SeekFrom};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Endian {
    Big,
    Little,
}

pub struct EndianReader<R> {
    inner: R,
    endian: Endian,
}

impl<R: Seek> Seek for EndianReader<R> {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        self.inner.seek(pos)
    }
}

impl<R: Read> EndianReader<R> {
    pub fn new(reader: R, endian: Endian) -> EndianReader<R> {
        EndianReader {
            inner: reader,
            endian,
        }
    }

    pub fn read_u16(&mut self) -> Result<u16> {
        let mut buf: [u8; 2] = [0; 2];
        self.inner.read_exact(&mut buf)?;

        let value = u16::from_bytes(buf);
        let ret = match self.endian {
            Endian::Big => u16::from_be(value),
            Endian::Little => u16::from_le(value),
        };
        Ok(ret)
    }

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
