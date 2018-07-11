use endian::Endian;
use ifd::{IFDIterator, IFD};
use std::io::{Error, ErrorKind, Read, Result, Seek};
const TIFF_LE: u16 = 0x4949;
const TIFF_BE: u16 = 0x4D4D;

pub struct Reader<R> {
    inner: R,
    order: Endian,
    ifds: Vec<IFD>,
}

impl<R: Read + Seek> Reader<R> {
    pub fn new(mut reader: R) -> Result<Reader<R>> {
        // Check order raw validation
        let mut order_bytes = [0, 0];
        reader.read_exact(&mut order_bytes)?;

        let order_raw = u16::to_be(u16::from_bytes(order_bytes));
        let order = match order_raw {
            TIFF_LE => Endian::Little,
            TIFF_BE => Endian::Big,
            _ => {
                return Err(Error::new(ErrorKind::InvalidInput, "Non recognized file"));
            }
        };

        // Valid magic number for tiff
        let mut tiff_magic_raw = [0, 0];
        reader.read_exact(&mut tiff_magic_raw)?;
        let tiff_magic = match order {
            Endian::Big => u16::from_be(u16::from_bytes(tiff_magic_raw)),
            Endian::Little => u16::from_le(u16::from_bytes(tiff_magic_raw)),
        };

        if tiff_magic != 42u16 {
            return Err(Error::new(ErrorKind::InvalidInput, "Non recognized file"));
        }

        // Read
        let mut offset_bytes: [u8; 4] = [0; 4];
        reader.read_exact(&mut offset_bytes)?;

        let offset = match order {
            Endian::Big => u32::from_be(u32::from_bytes(offset_bytes)),
            Endian::Little => u32::from_le(u32::from_bytes(offset_bytes)),
        };

        let ifds = IFDIterator::new(&mut reader, order, offset as usize).collect();

        Ok(Reader {
            inner: reader,
            order: order,
            ifds: ifds,
        })
    }

    pub fn ifds(&self) -> &Vec<IFD> {
        return &self.ifds;
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

        assert_eq!(read.order, Endian::Big);

        let elements = read.ifds();

        for el in elements {
            println!("Element: {:?}", el);
        }
        assert!(elements.len() > 0);
    }
}
