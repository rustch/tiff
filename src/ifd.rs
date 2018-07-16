use byteorder::{ByteOrder, ReadBytesExt};
use std::collections::HashMap;
use std::convert::From;
use std::io::{Read, Result, Seek, SeekFrom};
use std::iter::Iterator;
use std::marker::PhantomData;
use tag::Tag;

#[derive(Debug)]
pub enum IFDValue {
    Byte(Vec<u8>),
    Ascii(Vec<u8>),
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
    pub fn new_from_entry<R: Read + Seek>(reader: &mut R, entry: &IFDEntry) -> Result<IFDValue> {
        let mut short_buff: [u8; 4] = [0xFF; 4];

        match entry.value_type {
            1 => {
                let bytes = IFDValue::read_bytes(reader, entry)?;
                Ok(IFDValue::Byte(bytes))
            }
            _ => Ok(IFDValue::Undefined),
        }
    }

    fn read_bytes<R: Read + Seek>(reader: &mut R, entry: &IFDEntry) -> Result<Vec<u8>> {
        let mut short_buff: [u8; 4] = [0xFF; 4];

        if entry.count <= 4 {
            let buff = &mut short_buff[..entry.count as usize];
            reader.read_exact(buff)?;
            Ok(buff.to_vec())
        } else {
            reader.seek(SeekFrom::Start(entry.value_offset as u64))?;

            let vec: Vec<u8> = Vec::with_capacity(entry.count as usize);
            for i in 0..entry.count {
                reader.read_exact(&mut short_buff[..1])?;
                vec.push(short_buff[0]);
            }
            Ok(vec)
        }
    }

    fn read_ascii<R: Read + Seek>(reader: &mut R, entry: &IFDEntry) -> Result<Vec<u8>> {}
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

pub struct IFDIterator<'a, R: 'a, T> {
    reader: &'a mut R,
    first_entry: usize,
    position: usize,
    endian: PhantomData<T>,
}

impl<'a, R: Read + Seek, T: ByteOrder> IFDIterator<'a, R, T>
where
    R: 'a,
{
    pub fn new(reader: &'a mut R, first_ifd_offset: usize) -> IFDIterator<R, T> {
        reader.seek(SeekFrom::Start(0)).ok();

        IFDIterator {
            reader: reader,
            first_entry: first_ifd_offset,
            position: 0,
            endian: PhantomData,
        }
    }
}

impl<'a, R: Read + Seek, T: ByteOrder> Iterator for IFDIterator<'a, R, T> {
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
        let entry_count = self.reader.read_u16::<T>().ok()?;
        let mut map = HashMap::<Tag, IFDEntry>::new();
        for _i in 0..entry_count {
            // Tag
            let tag = self.reader.read_u16::<T>().ok()?;

            // Type
            let value_type_raw = self.reader.read_u16::<T>().ok()?;

            // Count
            let count = self.reader.read_u32::<T>().ok()?;
            let value_offset = self.reader.read_u32::<T>().ok()?;

            let tag_value = Tag::from(tag);
            let entry = IFDEntry {
                tag: tag_value,
                value_type: value_type_raw,
                count: count,
                value_offset: value_offset,
            };

            map.insert(tag_value, entry);
        }

        let next = self.reader.read_u32::<T>().ok()?;

        Some(IFD {
            entries: map,
            next: next as usize,
        })
    }
}
