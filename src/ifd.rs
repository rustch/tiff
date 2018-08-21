use endian::{Endian, EndianReader, Long, LongLong, Short};

use std::collections::hash_map::Keys;
use std::collections::HashMap;

use std::io::{Read, Seek, SeekFrom};
use tag::Tag;

error_chain!{
    foreign_links {
        Io(::std::io::Error);
        AsciiFormat(::std::string::FromUtf8Error);
    }
    errors {
        EncodingError
    }
}

/// A generic rational helper struct
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rational<T: Long> {
    pub num: T,
    pub denom: T,
}

/// A `TIFFValue` represents the primitives stores inside the
/// TIFF file format
#[derive(Debug)]
pub enum TIFFValue {
    Byte(Vec<u8>),
    Ascii(Vec<String>),
    Short(Vec<u16>),
    Long(Vec<u32>),
    Rational(Vec<Rational<u32>>),
    SByte(Vec<i8>),
    Undefined(Vec<u8>),
    SShort(Vec<i16>),
    SLong(Vec<i32>),
    SRational(Vec<Rational<i32>>),
    Float(Vec<f32>),
    Double(Vec<f64>),
}

impl TIFFValue {
    pub fn new_from_entry<R: Read + Seek>(
        reader: &mut R,
        entry: &IFDEntry,
        endian: Endian,
    ) -> Result<TIFFValue> {
        match entry.value_type {
            1 => {
                let bytes = TIFFValue::read_n_bytes(reader, entry, entry.count as usize)?;
                Ok(TIFFValue::Byte(bytes))
            }

            2 => {
                let values = TIFFValue::read_ascii(reader, entry)?;
                Ok(TIFFValue::Ascii(values))
            }

            3 => {
                let values = TIFFValue::read_short(reader, entry, endian)?;
                Ok(TIFFValue::Short(values))
            }

            4 => {
                let values = TIFFValue::read_long(reader, entry, endian)?;
                Ok(TIFFValue::Long(values))
            }

            5 => {
                let values = TIFFValue::read_rational(reader, entry, endian)?;
                Ok(TIFFValue::Rational(values))
            }

            6 => {
                let mut bytes = TIFFValue::read_n_bytes(reader, entry, entry.count as usize)?;
                let result = bytes.iter().map(|i| *i as i8).collect();
                Ok(TIFFValue::SByte(result))
            }

            8 => {
                let values = TIFFValue::read_short(reader, entry, endian)?;
                Ok(TIFFValue::SShort(values))
            }

            9 => {
                let values = TIFFValue::read_long(reader, entry, endian)?;
                Ok(TIFFValue::SLong(values))
            }
            10 => {
                let values = TIFFValue::read_rational(reader, entry, endian)?;
                Ok(TIFFValue::SRational(values))
            }
            11 => {
                let values: Vec<u32> = TIFFValue::read_long(reader, entry, endian)?;
                let result = values.iter().map(|i| f32::from_bits(*i)).collect();
                Ok(TIFFValue::Float(result))
            }
            12 => {
                let values: Vec<u64> = TIFFValue::read_long_long(reader, entry, endian)?;
                let result = values.iter().map(|i| f64::from_bits(*i)).collect();
                Ok(TIFFValue::Double(result))
            }
            _ => {
                let bytes = TIFFValue::read_n_bytes(reader, entry, entry.count as usize)?;
                Ok(TIFFValue::Undefined(bytes))
            }
        }
    }

    pub fn to_ifd_entry(self, tag: Tag, endian: Endian) -> Result<IFDEntry> {
        let value_type: u16 = match self {
            TIFFValue::Byte(_) => 1,
            TIFFValue::Ascii(_) => 2,
            TIFFValue::Short(_) => 3,
            TIFFValue::Long(_) => 4,
            TIFFValue::Rational(_) => 5,
            TIFFValue::SByte(_) => 6,
            TIFFValue::Undefined(_) => 7,
            TIFFValue::SShort(_) => 8,
            TIFFValue::SLong(_) => 9,
            TIFFValue::SRational(_) => 10,
            TIFFValue::Float(_) => 11,
            TIFFValue::Double(_) => 12,
        };

        let payload: (usize, Vec<u8>) = match self {
            TIFFValue::Byte(val) => (val.len(), val),
            TIFFValue::Ascii(val) => {
                if val.iter().all(|s| s[..].is_ascii()) {
                    return Err(ErrorKind::EncodingError.into());
                }

                let size = val.len();
                let content = val.into_iter().flat_map(|s| s.into_bytes()).collect();
                (size, content)
            }
            TIFFValue::Short(val) => {
                let len = val.len();
                let mut buff = Vec::new();

                for el in val {
                    buff.extend_from_slice(&el.to_bytes());
                }
                (len, buff)
            }
            TIFFValue::Long(val) => {
                let len = val.len();
                let mut buff = Vec::new();
                for el in val {
                    buff.extend_from_slice(&el.to_bytes());
                }
                (len, buff)
            }
            TIFFValue::Rational(val) => (val.len(), vec![]),
            TIFFValue::SByte(val) => {
                let len = val.len();
                let mut buff = Vec::new();
                for el in val {
                    buff.extend_from_slice(&el.to_bytes());
                }
                (len, buff)
            }
            TIFFValue::Undefined(val) => (val.len(), vec![]),
            TIFFValue::SShort(val) => {
                let len = val.len();
                let mut buff = Vec::new();
                for el in val {
                    buff.extend_from_slice(&el.to_bytes());
                }
                (len, buff)
            }
            TIFFValue::SLong(val) => {
                let len = val.len();
                let mut buff = Vec::new();
                for el in val {
                    buff.extend_from_slice(&el.to_bytes());
                }
                (len, buff)
            }
            TIFFValue::SRational(val) => (val.len(), vec![]),
            TIFFValue::Float(val) => (val.len(), vec![]),
            TIFFValue::Double(val) => (val.len(), vec![]),
            _ => return Err(ErrorKind::EncodingError.into()),
        };

        Ok(IFDEntry {
            value_offset: 0,
            count: payload.0 as u32,
            writing_payload: Some(payload.1),
            tag,
            value_type,
        })
    }

    fn read_n_bytes<R: Read + Seek>(
        reader: &mut R,
        entry: &IFDEntry,
        size: usize,
    ) -> Result<Vec<u8>> {
        if size <= 4 {
            let bytes = &entry.value_offset.to_bytes();
            Ok(bytes.to_vec())
        } else {
            reader.seek(SeekFrom::Start(u64::from(entry.value_offset)))?;
            let mut vec: Vec<u8> = vec![0; size];
            reader.read_exact(&mut vec)?;
            Ok(vec)
        }
    }

    fn read_ascii<R: Read + Seek>(reader: &mut R, entry: &IFDEntry) -> Result<Vec<String>> {
        let bytes = TIFFValue::read_n_bytes(reader, entry, entry.count as usize)?;

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
        let mut bytes = TIFFValue::read_n_bytes(reader, entry, size as usize)?;

        if endian == Endian::Big && size <= 4 {
            bytes.reverse()
        }

        let elements: Vec<T> = bytes
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
        let mut bytes = TIFFValue::read_n_bytes(reader, entry, size as usize)?;

        if endian == Endian::Big && size <= 4 {
            bytes.reverse()
        }

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
        let mut bytes = TIFFValue::read_n_bytes(reader, entry, size as usize)?;

        if endian == Endian::Big && size <= 8 {
            bytes.reverse()
        }

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
        let bytes = TIFFValue::read_n_bytes(reader, entry, size as usize)?;

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
}

/// An `IFDEntry` represents an **image file directory**
/// mentionned inside the tiff specification. This is the base
#[derive(Debug)]
pub struct IFDEntry {
    pub tag: Tag,
    pub value_type: u16,
    pub count: u32,
    pub value_offset: u32,
    pub writing_payload: Option<Vec<u8>>, // Use donly when writing
}

#[derive(Debug)]
pub struct IFD {
    read_entries: HashMap<Tag, IFDEntry>,
}

impl<'a> IFD {
    pub fn get_entry_from_tag(&self, tag: Tag) -> Option<&IFDEntry> {
        self.read_entries.get(&tag)
    }

    pub fn all_tags(&self) -> Keys<Tag, IFDEntry> {
        self.read_entries.keys()
    }
}

pub struct IFDIterator<'a, R: Read + Seek + 'a> {
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
                writing_payload: None,
            };

            map.insert(tag_value, entry);
        }

        let next: u32 = self.reader.read_long().ok()?;
        self.next_entry = next as usize;

        Some(IFD { read_entries: map })
    }
}
