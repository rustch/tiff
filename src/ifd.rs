use endian::{Endian, EndianReader, Long, LongLong, Short};
use std::collections::HashMap;
use std::convert::From;
use std::io::{Error, ErrorKind, Read, Result, Seek, SeekFrom};
use std::iter::Iterator;

use tag::Tag;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rational<T: Long> {
    pub num: T,
    pub denom: T,
}

#[derive(Debug)]
pub enum IFDValue {
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

impl IFDValue {
    pub fn new_from_entry<R: Read + Seek>(
        reader: &mut R,
        entry: &IFDEntry,
        endian: Endian,
    ) -> Result<IFDValue> {
        match entry.value_type {
            1 => {
                let bytes = IFDValue::read_n_bytes(reader, entry, entry.count as usize)?;
                Ok(IFDValue::Byte(bytes))
            }

            2 => {
                let values = IFDValue::read_ascii(reader, entry)?;
                Ok(IFDValue::Ascii(values))
            }

            3 => {
                let values = IFDValue::read_short(reader, entry, endian)?;
                Ok(IFDValue::Short(values))
            }

            4 => {
                let values = IFDValue::read_long(reader, entry, endian)?;
                Ok(IFDValue::Long(values))
            }

            5 => {
                let values = IFDValue::read_rational(reader, entry, endian)?;
                Ok(IFDValue::Rational(values))
            }

            6 => {
                let mut bytes = IFDValue::read_n_bytes(reader, entry, entry.count as usize)?;
                let result = bytes.iter().map(|i| *i as i8).collect();
                Ok(IFDValue::SByte(result))
            }

            8 => {
                let values = IFDValue::read_short(reader, entry, endian)?;
                Ok(IFDValue::SShort(values))
            }

            9 => {
                let values = IFDValue::read_long(reader, entry, endian)?;
                Ok(IFDValue::SLong(values))
            }
            10 => {
                let values = IFDValue::read_rational(reader, entry, endian)?;
                Ok(IFDValue::SRational(values))
            }
            11 => {
                let values: Vec<u32> = IFDValue::read_long(reader, entry, endian)?;
                let result = values.iter().map(|i| f32::from_bits(*i)).collect();
                Ok(IFDValue::Float(result))
            }
            12 => {
                let values: Vec<u64> = IFDValue::read_long_long(reader, entry, endian)?;
                let result = values.iter().map(|i| f64::from_bits(*i)).collect();
                Ok(IFDValue::Double(result))
            }
            _ => {
                let bytes = IFDValue::read_n_bytes(reader, entry, entry.count as usize)?;
                Ok(IFDValue::Undefined(bytes))
            }
        }
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
            reader.seek(SeekFrom::Start(entry.value_offset as u64))?;
            let mut vec: Vec<u8> = vec![0; size];
            reader.read_exact(&mut vec)?;
            Ok(vec)
        }
    }

    fn read_ascii<R: Read + Seek>(reader: &mut R, entry: &IFDEntry) -> Result<Vec<String>> {
        let bytes = IFDValue::read_n_bytes(reader, entry, entry.count as usize)?;

        // Splits by null cahracter
        bytes
            .split(|e| *e == '0' as u8)
            .map(|a| {
                String::from_utf8(a.to_vec())
                    .map_err(|_e| Error::new(ErrorKind::InvalidData, "Unexepcted String"))
            }).collect()
    }

    fn read_short<R: Read + Seek, T: Short>(
        reader: &mut R,
        entry: &IFDEntry,
        endian: Endian,
    ) -> Result<Vec<T>> {
        let mut conv_buff: [u8; 2] = [0; 2];
        let size = entry.count * 2;
        let mut bytes = IFDValue::read_n_bytes(reader, entry, size as usize)?;

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
        let mut bytes = IFDValue::read_n_bytes(reader, entry, size as usize)?;

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
        let mut bytes = IFDValue::read_n_bytes(reader, entry, size as usize)?;

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
        let bytes = IFDValue::read_n_bytes(reader, entry, size as usize)?;

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
}

#[derive(Debug)]
pub struct IFD {
    entries: HashMap<Tag, IFDEntry>,
    next: usize,
}

impl IFD {
    pub fn get_entry_from_tag(&self, tag: Tag) -> Option<&IFDEntry> {
        self.entries.get(&tag)
    }
}

pub struct IFDIterator<'a, R: Read + Seek + 'a> {
    reader: EndianReader<'a, R>,
    first_entry: usize,
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
            first_entry: first_ifd_offset,
            position: 0,
        }
    }
}

impl<'a, R: Read + Seek> Iterator for IFDIterator<'a, R> {
    type Item = IFD;

    fn next(&mut self) -> Option<IFD> {
        // Go to next entry
        let next = if self.position == 0 {
            SeekFrom::Start(self.first_entry as u64)
        } else {
            SeekFrom::Current(self.position as i64)
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
                count: count,
                value_offset: value_offset,
            };

            map.insert(tag_value, entry);
        }

        let next: u32 = self.reader.read_long().ok()?;

        Some(IFD {
            entries: map,
            next: next as usize,
        })
    }
}
