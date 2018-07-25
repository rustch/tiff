use endian::Endian;
use ifd::{IFDEntry, IFDIterator, IFDValue, IFD};
use std::io::{Error, ErrorKind, Read, Result, Seek};
use tag::TIFFTag;
const TIFF_LE: u16 = 0x4949;
const TIFF_BE: u16 = 0x4D4D;

pub struct TIFF<R> {
    inner: R,
    ifds: Vec<IFD>,
    endian: Endian,
}

impl<R: Read + Seek> TIFF<R> {
    /// Creates a new TIFF reader from the input `Read` type.
    pub fn new(mut reader: R) -> Result<TIFF<R>> {
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
            Ok(TIFF {
                inner: reader,
                ifds: ifds,
                endian: order,
            })
        }
    }

    /// Returns the endianness of the processed input.
    pub fn endianness(&self) -> Endian {
        self.endian
    }

    /// Returns a reference to the IFD directories contained inside the reader.
    pub fn directory_entries(&self) -> &Vec<IFD> {
        &self.ifds
    }

    /// Look for a specific tag in all IFDS.
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

    /// Access to the IFDS contained inside the image
    pub fn ifds(&self) -> &Vec<IFD> {
        &self.ifds
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use endian::Endian;
    use ifd::Rational;
    use std::io::Cursor;
    use tag::*;

    macro_rules! ensure_field {
        ($read:expr, $type:ty) => {
            $read
                .get_tiff_value::<$type>()
                .expect(stringify!("We expect to be able to read" $type))
        };
    }
    #[test]
    fn test_sample_be() {
        let bytes: &[u8] = include_bytes!("../samples/arbitro_be.tiff");
        let mut cursor = Cursor::new(bytes);
        let mut read = TIFF::new(&mut cursor).unwrap();
        // println!("IFD {:?}", read.ifds());
        assert_eq!(read.endianness(), Endian::Big);

        let image_width = ensure_field!(read, ImageWidth);
        assert_eq!(image_width.0, 174);

        let photometric_interpretation = ensure_field!(read, PhotometricInterpretation);
        assert_eq!(photometric_interpretation, PhotometricInterpretation::RGB);

        let strip_offsets = ensure_field!(read, StripOffsets);
        assert_eq!(strip_offsets.0.len(), 1);
        assert_eq!(strip_offsets.0[0], 8);

        let samples_per_pixel = ensure_field!(read, SamplesPerPixel);
        assert_eq!(samples_per_pixel.0, 4);

        let rows_per_strip = ensure_field!(read, RowsPerStrip);
        assert_eq!(rows_per_strip.0, 38);

        let strip_byte_counts = ensure_field!(read, StripByteCounts);
        assert_eq!(strip_byte_counts.0.len(), 1);
        assert_eq!(strip_byte_counts.0[0], 6391);

        let bits_per_sample = ensure_field!(read, BitsPerSample);
        assert_eq!(bits_per_sample.0, vec![8, 8, 8, 8]);
    }

    #[test]
    fn test_sample_le() {
        let bytes: &[u8] = include_bytes!("../samples/picoawards_le.tiff");
        let mut cursor = Cursor::new(bytes);
        let mut read = TIFF::new(&mut cursor).unwrap();
        // println!("IFD {:?}", read.ifds());
        assert_eq!(read.endianness(), Endian::Little);

        let image_width = ensure_field!(read, ImageWidth);
        assert_eq!(image_width.0, 436);

        let photometric_interpretation = ensure_field!(read, PhotometricInterpretation);
        assert_eq!(photometric_interpretation, PhotometricInterpretation::RGB);

        // let strip_offsets = ensure_field!(read, StripOffsets);
        //  et!(value.0, 8);
        //

        let samples_per_pixel = ensure_field!(read, SamplesPerPixel);
        assert_eq!(samples_per_pixel.0, 3);

        let rows_per_strip = ensure_field!(read, RowsPerStrip);
        assert_eq!(rows_per_strip.0, 9);

        // let strip_byte_counts = ensure_field!(read, StripByteCounts);
        //  et!(value.0, 6391);
        //

        let bits_per_sample = ensure_field!(read, BitsPerSample);
        assert_eq!(bits_per_sample.0, vec![8, 8, 8]);

        let x_resolution = ensure_field!(read, XResolution);
        assert_eq!(x_resolution.0, Rational { num: 96, denom: 1 });

        let y_resolution = ensure_field!(read, YResolution);
        assert_eq!(y_resolution.0, Rational { num: 96, denom: 1 });

        let resolution_unit = ensure_field!(read, ResolutionUnit);
        assert_eq!(resolution_unit, ResolutionUnit::Inch);

        let predictor = ensure_field!(read, Predictor);
        assert_eq!(predictor, Predictor::None);

        let planar = ensure_field!(read, PlanarConfiguration);
        assert_eq!(planar, PlanarConfiguration::Chunky);

        let subfile = ensure_field!(read, NewSubfileType);
        assert_eq!(false, subfile.is_reduced_image());
    }
}
