use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use ifd::{IFDIterator, IFD};
use std::io::{Error, ErrorKind, Read, Result, Seek};

const TIFF_LE: u16 = 0x4949;
const TIFF_BE: u16 = 0x4D4D;

pub struct Reader<R> {
    inner: R,
    ifds: Vec<IFD>,
    is_le: bool,
}

impl<R: Read + Seek> Reader<R> {
    /// Creates a new TIFF reader from the input `Read` type.
    pub fn new(mut reader: R) -> Result<Reader<R>> {
        // Check order raw validation

        let order_raw = reader.read_u16::<BigEndian>()?;
        let is_little_endian = match order_raw {
            TIFF_LE => true,
            TIFF_BE => false,
            _ => {
                return Err(Error::new(ErrorKind::InvalidInput, "Non recognized file"));
            }
        };

        // Valid magic number for tiff
        let tiff_magic = match is_little_endian {
            false => reader.read_u16::<BigEndian>()?,
            true => reader.read_u16::<LittleEndian>()?,
        };

        if tiff_magic != 42u16 {
            return Err(Error::new(ErrorKind::InvalidInput, "Non recognized file"));
        }

        // Read
        let offset = match is_little_endian {
            false => reader.read_u32::<BigEndian>()?,
            true => reader.read_u32::<LittleEndian>()?,
        };

        let ifds: Vec<IFD>;
        if is_little_endian {
            let iter: IFDIterator<R, LittleEndian> = IFDIterator::new(&mut reader, offset as usize);
            ifds = iter.collect();
        } else {
            let iter: IFDIterator<R, BigEndian> = IFDIterator::new(&mut reader, offset as usize);
            ifds = iter.collect();
        }

        Ok(Reader {
            inner: reader,
            ifds: ifds,
            is_le: is_little_endian,
        })
    }

    pub fn is_little_endian(&self) -> bool {
        self.is_le
    }

    pub fn is_big_endian(&self) -> bool {
        !self.is_little_endian()
    }

    pub fn directory_entries(&self) -> &Vec<IFD> {
        &self.ifds
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_iter_creation() {
        let bytes: &[u8] = include_bytes!("../samples/arbitro_be.tiff");
        let mut cursor = Cursor::new(bytes);
        let read = Reader::new(&mut cursor).unwrap();

        assert_eq!(read.is_little_endian(), false);
    }

}
