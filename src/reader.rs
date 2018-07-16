use endian::Endian;
use ifd::{IFDEntry, IFDIterator, IFDValue, IFD};
use std::io::{Error, ErrorKind, Read, Result, Seek, SeekFrom};
use tag::Tag;
const TIFF_LE: u16 = 0x4949;
const TIFF_BE: u16 = 0x4D4D;

pub struct Reader<R> {
    inner: R,
    ifds: Vec<IFD>,
    endian: Endian,
}

impl<R: Read + Seek> Reader<R> {
    /// Creates a new TIFF reader from the input `Read` type.
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

        let ifds: Vec<IFD> = IFDIterator::new(&mut reader, offset as usize, order).collect();

        Ok(Reader {
            inner: reader,
            ifds: ifds,
            endian: order,
        })
    }

    pub fn endianness(&self) -> Endian {
        self.endian
    }

    pub fn directory_entries(&self) -> &Vec<IFD> {
        &self.ifds
    }

    pub fn value_from_tag(&mut self, tag: Tag) -> Option<IFDValue> {
        // Check if we have an entry inside any of the directory

        let ifd_entry: &IFDEntry;
        ifd_entry = self
            .ifds
            .iter()
            .flat_map(|entry| entry.get_entry_from_tag(tag))
            .next()?;

        //  If yes, we construct the value
        self.inner
            .seek(SeekFrom::Start(ifd_entry.value_offset as u64))
            .ok();

        let value = IFDValue::new_from_entry(&mut self.inner, ifd_entry).ok()?;
        Some(value)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use endian::Endian;
    use std::io::Cursor;
    use tag::Tag;

    #[test]
    fn test_basic_usage() {
        let bytes: &[u8] = include_bytes!("../samples/arbitro_be.tiff");
        let mut cursor = Cursor::new(bytes);
        let mut read = Reader::new(&mut cursor).unwrap();

        assert_eq!(read.endianness(), Endian::Big);

        if let Some(value) = read.value_from_tag(Tag::ImageWidth) {
            match value {
                IFDValue::Short(_) => assert!(true),
                IFDValue::Long(_) => assert!(true),
                _ => assert!(false, "Invalid value"),
            }
        } else {
            assert!(false, "We expect to be able to read image width");
        }
    }

}
