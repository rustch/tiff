use endian::Endian;
use ifd::{IFDEntry, IFDIterator, IFDValue, IFD};
use std::io::{Error, ErrorKind, Read, Result, Seek};
use tag::TIFFTag;
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
        if ifds.len() < 1 {
            Err(Error::new(
                ErrorKind::InvalidInput,
                "A TIFF file shoudl have at least one IFD",
            ))
        } else {
            Ok(Reader {
                inner: reader,
                ifds: ifds,
                endian: order,
            })
        }
    }

    pub fn endianness(&self) -> Endian {
        self.endian
    }

    pub fn directory_entries(&self) -> &Vec<IFD> {
        &self.ifds
    }

    pub fn get_tiff_value<T: TIFFTag>(&mut self) -> Option<T> {
        // Check if we have an entry inside any of the directory

        let tag = T::tag();
        let ifd_entry: &IFDEntry;
        ifd_entry = self
            .ifds
            .iter()
            .flat_map(|entry| entry.get_entry_from_tag(tag))
            .next()?;

        let value = IFDValue::new_from_entry(&mut self.inner, ifd_entry, self.endian).ok()?;
        T::new_from_value(&value)
    }

    pub fn ifds(&self) -> &Vec<IFD> {
        &self.ifds
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use endian::Endian;
    use std::io::Cursor;
    use tag::*;

    #[test]
    fn test_sample_be() {
        let bytes: &[u8] = include_bytes!("../samples/arbitro_be.tiff");
        let mut cursor = Cursor::new(bytes);
        let mut read = Reader::new(&mut cursor).unwrap();
        // println!("IFD {:?}", read.ifds());
        assert_eq!(read.endianness(), Endian::Big);

        if let Some(value) = read.get_tiff_value::<ImageWidth>() {
            assert_eq!(value.0, 174);
        } else {
            assert!(false, "We expect to be able to read image width");
        }

        if let Some(value) = read.get_tiff_value::<PhotometricInterpretation>() {
            assert_eq!(value, PhotometricInterpretation::RGB);
        } else {
            assert!(false, "We expect to be able to PhotometricInterpretation");
        }

        if let Some(value) = read.get_tiff_value::<StripOffsets>() {
            assert_eq!(value.0, 8);
        } else {
            assert!(false, "We expect to be able to StripOffsets");
        }

        if let Some(value) = read.get_tiff_value::<SamplesPerPixel>() {
            assert_eq!(value.0, 4);
        } else {
            assert!(false, "We expect to be able to SamplesPerPixel");
        }

        if let Some(value) = read.get_tiff_value::<RowsPerStrip>() {
            assert_eq!(value.0, 38);
        } else {
            assert!(false, "We expect to be able to RowsPerStrip");
        }

        if let Some(value) = read.get_tiff_value::<StripByteCounts>() {
            assert_eq!(value.0, 6391);
        } else {
            assert!(false, "We expect to be able to StripByteCounts");
        }

        if let Some(value) = read.get_tiff_value::<BitsPerSample>() {
            assert_eq!(value.0, vec![8, 8, 8, 8]);
        } else {
            assert!(false, "We expect to be able to BitsPerSample");
        }
    }

    #[test]
    fn test_sample_le() {
        let bytes: &[u8] = include_bytes!("../samples/picoawards_le.tiff");
        let mut cursor = Cursor::new(bytes);
        let mut read = Reader::new(&mut cursor).unwrap();
        // println!("IFD {:?}", read.ifds());
        assert_eq!(read.endianness(), Endian::Little);

        if let Some(value) = read.get_tiff_value::<ImageWidth>() {
            assert_eq!(value.0, 436);
        } else {
            assert!(false, "We expect to be able to read image width");
        }

        if let Some(value) = read.get_tiff_value::<PhotometricInterpretation>() {
            assert_eq!(value, PhotometricInterpretation::RGB);
        } else {
            assert!(false, "We expect to be able to PhotometricInterpretation");
        }

        // if let Some(value) = read.get_tiff_value::<StripOffsets>() {
        //     assert_eq!(value.0, 8);
        // } else {
        //     assert!(false, "We expect to be able to StripOffsets");
        // }

        if let Some(value) = read.get_tiff_value::<SamplesPerPixel>() {
            assert_eq!(value.0, 3);
        } else {
            assert!(false, "We expect to be able to SamplesPerPixel");
        }

        if let Some(value) = read.get_tiff_value::<RowsPerStrip>() {
            assert_eq!(value.0, 9);
        } else {
            assert!(false, "We expect to be able to RowsPerStrip");
        }

        // if let Some(value) = read.get_tiff_value::<StripByteCounts>() {
        //     assert_eq!(value.0, 6391);
        // } else {
        //     assert!(false, "We expect to be able to StripByteCounts");
        // }

        if let Some(value) = read.get_tiff_value::<BitsPerSample>() {
            assert_eq!(value.0, vec![8, 8, 8]);
        } else {
            assert!(false, "We expect to be able to BitsPerSample");
        }
    }
}
