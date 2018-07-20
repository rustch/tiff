use endian::{Endian, EndianReader};
use std::collections::HashMap;
use std::convert::From;
use std::io::{Error, ErrorKind, Read, Result, Seek, SeekFrom};
use std::iter::Iterator;
use tag::Tag;

#[derive(Debug)]
pub enum IFDValue {
    Byte(Vec<u8>),
    Ascii(Vec<String>),
    Short(Vec<u16>),
    Long(Vec<u32>),
    Rational,
    SByte(Vec<i8>),
    Undefined,
    SShort(Vec<i16>),
    SLong(Vec<i32>),
    SRational,
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
            _ => Ok(IFDValue::Undefined),
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
            let mut vec: Vec<u8> = Vec::with_capacity(size);
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
            })
            .collect()
    }

    fn read_short<R: Read + Seek>(
        reader: &mut R,
        entry: &IFDEntry,
        endian: Endian,
    ) -> Result<Vec<u16>> {
        let mut conv_buff: [u8; 2] = [0; 2];
        let size = entry.count * 2;
        let mut bytes = IFDValue::read_n_bytes(reader, entry, size as usize)?;
        if endian == Endian::Big {
            bytes.reverse()
        }

        let elements: Vec<u16> = bytes
            .chunks(2)
            .map(|e| {
                conv_buff.copy_from_slice(e);

                let bytes = u16::from_bytes(conv_buff);
                endian.adjust_u16(bytes)
            })
            .collect();

        Ok(elements)
    }

    fn read_long<R: Read + Seek>(
        reader: &mut R,
        entry: &IFDEntry,
        endian: Endian,
    ) -> Result<Vec<u32>> {
        let mut conv_buff: [u8; 4] = [0; 4];
        let size = entry.count * 4;
        let mut bytes = IFDValue::read_n_bytes(reader, entry, size as usize)?;

        if endian == Endian::Big {
            bytes.reverse()
        }

        let elements: Vec<u32> = bytes
            .chunks(4)
            .map(|e| {
                conv_buff.copy_from_slice(e);
                let bytes = u32::from_bytes(conv_buff);
                endian.adjust_u32(bytes)
            })
            .collect();
        Ok(elements)
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
        let entry_count = self.reader.read_u16().ok()?;
        if entry_count < 1 {
            return None;
        }

        let mut map = HashMap::<Tag, IFDEntry>::new();
        for _i in 0..entry_count {
            // Tag
            let tag = self.reader.read_u16().ok()?;

            // Type
            let value_type_raw = self.reader.read_u16().ok()?;

            // Count
            let count = self.reader.read_u32().ok()?;
            let value_offset = self.reader.read_u32().ok()?;

            let tag_value = Tag::from(tag);
            let entry = IFDEntry {
                tag: tag_value,
                value_type: value_type_raw,
                count: count,
                value_offset: value_offset,
            };

            map.insert(tag_value, entry);
        }

        let next = self.reader.read_u32().ok()?;

        Some(IFD {
            entries: map,
            next: next as usize,
        })
    }
}
