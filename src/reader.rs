use iterator::IFDIterator;
use std::io::{Read, Result, Error, ErrorKind, BufReader, Seek};
use endian::Endian;

const TIFF_LE: u16 = 0x4949;
const TIFF_BE: u16 = 0x4D4D;

pub struct Reader<R> {
    inner: BufReader<R>,
    order: Endian,
    first_ifd_offset: u64,
}

impl<R: Read + Seek> Reader<R> {

    pub fn new(reader: R) -> Result<Reader<R>> {

        let mut breader = BufReader::new(reader);
        let mut order_raw = [0, 0];
        breader.read_exact(&mut order_raw)?;
        let magic_number = u16::to_be(u16::from_bytes(order_raw));

        // TIFF
        let order = match magic_number {
            TIFF_LE => Endian::Little,
            TIFF_BE => Endian::Big,
            _ => {
                return Err(Error::new(ErrorKind::InvalidInput, "Non recognized file"));
            }
        };

        // Valid magic number for tiff
        let mut tiff_magic_raw = [0,0];
        breader.read_exact(&mut tiff_magic_raw)?;
        let tiff_magic = match order {
            Endian::Big => u16::from_be(u16::from_bytes(tiff_magic_raw)),
            Endian::Little => u16::from_le(u16::from_bytes(tiff_magic_raw)),
        };

        if tiff_magic != 42u16 {
            return Err(Error::new(ErrorKind::InvalidInput, "Non recognized file"));
        }

        // Read
        let mut offset_bytes: [u8; 4] = [0; 4];
        breader.read_exact(&mut offset_bytes)?;
        
        let offset = match order {
            Endian::Big => { u32::from_be(u32::from_bytes(offset_bytes)) }
            Endian::Little => { u32::from_le(u32::from_bytes(offset_bytes)) }
        };

        Ok(Reader {
            inner: breader,
            order: order,
            first_ifd_offset: offset as u64
        })
    }

    pub fn iter() -> IFDIterator<R> {

    }
}