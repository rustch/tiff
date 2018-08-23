use endian::{Endian, EndianReader, Long, LongLong, Short};
use std::io::{Read, Seek, SeekFrom};

use std::iter::Iterator;

use super::{TIFF_BE, TIFF_LE};
use tag::Field;
use value::{Rational, TIFFValue};

use std::collections::HashMap;
use tag::Tag;

/// An `IFDEntry` represents an **image file directory**
/// mentionned inside the tiff specification. This is the base
#[derive(Debug, PartialEq)]
struct IFDEntry {
    tag: Tag,
    value_type: u16,
    count: u32,
    value_offset: u32,
}

#[derive(Debug)]
struct IFD {
    read_entries: HashMap<Tag, IFDEntry>,
}

impl<'a> IFD {
    fn get_entry_from_tag(&self, tag: Tag) -> Option<&IFDEntry> {
        self.read_entries.get(&tag)
    }

    fn all_tags(&self) -> Vec<Tag> {
        self.read_entries.keys().cloned().collect()
    }
}

struct IFDIterator<'a, R: Read + Seek + 'a> {
    reader: EndianReader<'a, R>,
    next_entry: usize,
    position: usize,
}

impl<'a, R: Read + Seek> IFDIterator<'a, R>
where
    R: 'a,
{
    pub fn new(reader: &'a mut R, first_ifd_offset: usize, endian: Endian) -> IFDIterator<R> {
        reader.seek(SeekFrom::Start(0)).ok();

        IFDIterator {
            reader: EndianReader::new(reader, endian),
            next_entry: first_ifd_offset,
            position: 0,
        }
    }
}

impl<'a, R: Read + Seek> Iterator for IFDIterator<'a, R> {
    type Item = IFD;

    fn next(&mut self) -> Option<IFD> {
        // Go to next entry
        let next = if self.position == 0 {
            SeekFrom::Start(self.next_entry as u64)
        } else {
            SeekFrom::Current(self.next_entry as i64)
        };

        self.position = self.reader.seek(next).ok()? as usize;

        // Read Count
        let entry_count: u16 = self.reader.read_short().ok()?;
        if entry_count < 1 {
            return None;
        }

        let mut map = HashMap::<Tag, IFDEntry>::new();
        for _i in 0..entry_count {
            // Tag
            let tag: u16 = self.reader.read_short().ok()?;

            // Type
            let value_type_raw: u16 = self.reader.read_short().ok()?;

            // Count
            let count: u32 = self.reader.read_long().ok()?;
            let value_offset: u32 = self.reader.read_long().ok()?;

            let tag_value = Tag::from(tag);
            let entry = IFDEntry {
                tag: tag_value,
                value_type: value_type_raw,
                count,
                value_offset,
            };

            map.insert(tag_value, entry);
        }

        let next: u32 = self.reader.read_long().ok()?;
        self.next_entry = next as usize;

        Some(IFD { read_entries: map })
    }
}

error_chain!{
    foreign_links {
        Io(::std::io::Error);
        AsciiFormat(::std::string::FromUtf8Error);
    }
    errors {
        InvalidTIFFFile(v: &'static str) {
            description("Invalid TIFF file"),
            display("INvalid TIFF File: {}", v),
        }

        DirectoryIndexOutOfBounds
    }
}

pub struct TIFFReader<R> {
    inner: R,
    ifds: Vec<IFD>,
    endian: Endian,
    current_directory_index: usize,
}

impl<R: Read + Seek> TIFFReader<R> {
    /// Creates a new TIFF reader from the input `Read` type.
    pub fn new(mut reader: R) -> Result<TIFFReader<R>> {
        let mut header_bytes: [u8; 8] = Default::default();
        reader.read_exact(&mut header_bytes)?;

        let mut word_buff: [u8; 2] = Default::default();
        word_buff.copy_from_slice(&header_bytes[0..2]);

        let order_raw = u16::to_be(u16::from_ne_bytes(word_buff));
        let order = match order_raw {
            TIFF_LE => Endian::Little,
            TIFF_BE => Endian::Big,
            _ => {
                return Err(ErrorKind::InvalidTIFFFile("Invalid magic endian bytes").into());
            }
        };

        // Valid magic number for tiff
        word_buff.copy_from_slice(&header_bytes[2..4]);
        let tiff_magic = match order {
            Endian::Big => u16::from_be_bytes(word_buff),
            Endian::Little => u16::from_le_bytes(word_buff),
        };

        if tiff_magic != 42u16 {
            return Err(ErrorKind::InvalidTIFFFile("Invalid magic byte").into());
        }

        // Read
        let mut offset_bytes: [u8; 4] = Default::default();
        offset_bytes.copy_from_slice(&header_bytes[4..8]);

        let offset = match order {
            Endian::Big => u32::from_be_bytes(offset_bytes),
            Endian::Little => u32::from_le_bytes(offset_bytes),
        };

        let ifds: Vec<IFD> = IFDIterator::new(&mut reader, offset as usize, order).collect();
        if ifds.is_empty() {
            Err(ErrorKind::InvalidTIFFFile("TIFF file should have one least one directory").into())
        } else {
            Ok(TIFFReader {
                inner: reader,
                ifds,
                endian: order,
                current_directory_index: 0,
            })
        }
    }

    /// Returns the number of available directories
    pub fn directories_count(&self) -> usize {
        self.ifds.len()
    }

    /// Returns the endianness of the processed input.
    pub fn endianness(&self) -> Endian {
        self.endian
    }

    /// Look for a specific tag in all IFDS.
    pub fn get_directory_field<T: Field>(&mut self) -> Option<T> {
        // Check if we have an entry inside any of the directory
        let tag = T::tag();
        let value = self.get_directory_value_from_tag(tag)?;
        T::decode_from_value(&value)
    }

    /// Read the value from the reader corresponding to a tag
    pub fn get_directory_value_from_tag(&mut self, tag: Tag) -> Option<TIFFValue> {
        let ifd_entry = self.ifds[self.current_directory_index].get_entry_from_tag(tag)?;
        TIFFValue::new_from_entry(&mut self.inner, ifd_entry, self.endian).ok()
    }

    /// Returns the list of tags included in the current directory
    pub fn get_directory_tags(&self) -> Vec<Tag> {
        self.ifds[self.current_directory_index].all_tags()
    }

    /// Set the current reading TIFF directory
    pub fn set_directory_index(&mut self, index: usize) {
        if index > self.ifds.len() - 1 {
            panic!("Invalid directory index")
        }
        self.current_directory_index = index;
    }
}

fn read_n_bytes<R: Read + Seek>(
    reader: &mut R,
    entry: &IFDEntry,
    size: usize,
    endian: Endian,
) -> Result<Vec<u8>> {
    if size <= 4 {
        // We need to extract data from value_offset
        let mut bytes = endian.long_adjusted(entry.value_offset).to_vec();
        bytes.truncate(size);
        // let bytes = entry.value_offset.to_be_bytes()[4 - size..].to_vec();
        Ok(bytes)
    } else {
        reader.seek(SeekFrom::Start(u64::from(entry.value_offset)))?;
        let mut vec: Vec<u8> = vec![0; size];
        reader.read_exact(&mut vec)?;
        Ok(vec)
    }
}

fn read_ascii<R: Read + Seek>(
    reader: &mut R,
    entry: &IFDEntry,
    endian: Endian,
) -> Result<Vec<String>> {
    let bytes = read_n_bytes(reader, entry, entry.count as usize, endian)?;

    // Splits by null cahracter
    bytes
        .split(|e| *e == b'0')
        .map(|a| String::from_utf8(a.to_vec()).map_err(|e| ErrorKind::AsciiFormat(e).into()))
        .collect()
}

fn read_short<R: Read + Seek, T: Short>(
    reader: &mut R,
    entry: &IFDEntry,
    endian: Endian,
) -> Result<Vec<T>> {
    let mut conv_buff: [u8; 2] = [0; 2];
    let size = entry.count * 2;
    let bytes = read_n_bytes(reader, entry, size as usize, endian)?;

    let elements = bytes
        .chunks(2)
        .map(|e| {
            conv_buff.copy_from_slice(e);
            endian.short_from_bytes::<T>(conv_buff)
        }).collect();

    Ok(elements)
}

fn read_long<R: Read + Seek, T: Long>(
    reader: &mut R,
    entry: &IFDEntry,
    endian: Endian,
) -> Result<Vec<T>> {
    let mut conv_buff: [u8; 4] = [0; 4];
    let size = entry.count * 4;
    let bytes = read_n_bytes(reader, entry, size as usize, endian)?;

    let elements: Vec<T> = bytes
        .chunks(4)
        .map(|e| {
            conv_buff.copy_from_slice(e);
            endian.long_from_bytes::<T>(conv_buff)
        }).collect();
    Ok(elements)
}

fn read_long_long<R: Read + Seek, T: LongLong>(
    reader: &mut R,
    entry: &IFDEntry,
    endian: Endian,
) -> Result<Vec<T>> {
    let mut conv_buff: [u8; 8] = [0; 8];
    let size = entry.count * 8;
    let bytes = read_n_bytes(reader, entry, size as usize, endian)?;

    let elements: Vec<T> = bytes
        .chunks(8)
        .map(|e| {
            conv_buff.copy_from_slice(e);
            endian.longlong_from_bytes::<T>(conv_buff)
        }).collect();
    Ok(elements)
}

fn read_rational<R: Read + Seek, T: Long>(
    reader: &mut R,
    entry: &IFDEntry,
    endian: Endian,
) -> Result<Vec<Rational<T>>> {
    let size = entry.count * 8;
    let mut conv_buff: [u8; 4] = [0; 4];
    let bytes = read_n_bytes(reader, entry, size as usize, endian)?;

    let elements: Vec<T> = bytes
        .chunks(4)
        .map(|e| {
            conv_buff.copy_from_slice(e);
            endian.long_from_bytes::<T>(conv_buff)
        }).collect();

    Ok(elements
        .chunks(2)
        .map(|e| Rational {
            num: e[0],
            denom: e[1],
        }).collect())
}

impl TIFFValue {
    fn new_from_entry<R: Read + Seek>(
        reader: &mut R,
        entry: &IFDEntry,
        endian: Endian,
    ) -> Result<TIFFValue> {
        match entry.value_type {
            1 => {
                let bytes = read_n_bytes(reader, entry, entry.count as usize, endian)?;
                Ok(TIFFValue::Byte(bytes))
            }

            2 => {
                let values = read_ascii(reader, entry, endian)?;
                Ok(TIFFValue::Ascii(values))
            }

            3 => {
                let values = read_short(reader, entry, endian)?;
                Ok(TIFFValue::Short(values))
            }

            4 => {
                let values = read_long(reader, entry, endian)?;
                Ok(TIFFValue::Long(values))
            }

            5 => {
                let values = read_rational(reader, entry, endian)?;
                Ok(TIFFValue::Rational(values))
            }

            6 => {
                let mut bytes = read_n_bytes(reader, entry, entry.count as usize, endian)?;
                let result = bytes.iter().map(|i| *i as i8).collect();
                Ok(TIFFValue::SByte(result))
            }

            8 => {
                let values = read_short(reader, entry, endian)?;
                Ok(TIFFValue::SShort(values))
            }

            9 => {
                let values = read_long(reader, entry, endian)?;
                Ok(TIFFValue::SLong(values))
            }
            10 => {
                let values = read_rational(reader, entry, endian)?;
                Ok(TIFFValue::SRational(values))
            }
            11 => {
                let values: Vec<u32> = read_long(reader, entry, endian)?;
                let result = values.iter().map(|i| f32::from_bits(*i)).collect();
                Ok(TIFFValue::Float(result))
            }
            12 => {
                let values: Vec<u64> = read_long_long(reader, entry, endian)?;
                let result = values.iter().map(|i| f64::from_bits(*i)).collect();
                Ok(TIFFValue::Double(result))
            }
            _ => {
                let bytes = read_n_bytes(reader, entry, entry.count as usize, endian)?;
                Ok(TIFFValue::Undefined(bytes))
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use endian::Endian;
    use std::io::Cursor;
    use tag::*;
    use value::Rational;

    macro_rules! ensure_field {
        ($read:expr, $type:ty) => {
            $read
                .get_directory_field::<$type>()
                .expect(stringify!("We expect to be able to read" $type))
        };
    }

    #[test]
    fn test_iterator() {
        let bytes: &[u8] = include_bytes!("../samples/arbitro_be.tiff");
        let mut cursor = Cursor::new(bytes);
        let mut iter = IFDIterator::new(&mut cursor, 0x1900, Endian::Big);

        let first_dict = iter.next().unwrap();
        let entry = first_dict.get_entry_from_tag(Tag::ImageWidth).unwrap();
        assert_eq!(entry.value_offset, 11403264);
    }

    #[test]
    fn test_sample_be() {
        let bytes: &[u8] = include_bytes!("../samples/arbitro_be.tiff");
        let mut cursor = Cursor::new(bytes);
        let mut read = TIFFReader::new(&mut cursor).unwrap();
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
        let mut read = TIFFReader::new(&mut cursor).unwrap();
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

    #[test]
    fn test_sample_other() {
        let bytes: &[u8] = include_bytes!("../samples/ycbcr-cat.tif");
        let mut cursor = Cursor::new(bytes);
        let mut read = TIFFReader::new(&mut cursor).unwrap();
        // println!("IFD {:?}", read.ifds());
        assert_eq!(read.endianness(), Endian::Big);

        let image_width = ensure_field!(read, ImageWidth);
        assert_eq!(image_width.0, 250);

        let image_length = ensure_field!(read, ImageLength);
        assert_eq!(image_length.0, 325);

        let photometric_interpretation = ensure_field!(read, PhotometricInterpretation);
        assert_eq!(photometric_interpretation, PhotometricInterpretation::YCbCr);

        let samples_per_pixel = ensure_field!(read, SamplesPerPixel);
        assert_eq!(samples_per_pixel.0, 3);

        let rows_per_strip = ensure_field!(read, RowsPerStrip);
        assert_eq!(rows_per_strip.0, 10);

        // let strip_byte_counts = ensure_field!(read, StripByteCounts);
        //  et!(value.0, 6391);
        //

        let bits_per_sample = ensure_field!(read, BitsPerSample);
        assert_eq!(bits_per_sample.0, vec![8, 8, 8]);

        let planar = ensure_field!(read, PlanarConfiguration);
        assert_eq!(planar, PlanarConfiguration::Chunky);
    }
}
