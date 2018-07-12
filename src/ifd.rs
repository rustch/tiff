use byteorder::{ByteOrder, ReadBytesExt};
use std::collections::HashMap;
use std::convert::From;
use std::io::{Read, Seek, SeekFrom};
use std::iter::Iterator;
use std::marker::PhantomData;
use tag::Tag;

pub enum IFDValueBag {
    Byte(Vec<u8>),
    Ascii(Vec<u8>),
    Short,
    Long,
    Rational,
    SByte,
    Undefined,
    SShort,
    SLong,
    SRational,
    Float,
    Double,
}
#[derive(Debug, Copy, Clone)]
pub enum IFDValueType {
    Byte,
    Ascii,
    Short,
    Long,
    Rational,
    SByte,
    Undefined,
    SShort,
    SLong,
    SRational,
    Float,
    Double,
}

impl IFDValueType {
    fn from_int(value: u16) -> IFDValueType {
        match value {
            1 => IFDValueType::Byte,
            2 => IFDValueType::Ascii,
            3 => IFDValueType::Short,
            4 => IFDValueType::Long,
            5 => IFDValueType::Rational,
            6 => IFDValueType::SByte,
            7 => IFDValueType::Undefined,
            8 => IFDValueType::SShort,
            9 => IFDValueType::SLong,
            10 => IFDValueType::SRational,
            11 => IFDValueType::Float,
            12 => IFDValueType::Double,
            _ => IFDValueType::Undefined,
        }
    }
}

/// An `IFDEntry` represents an **image file directory**
/// mentionned inside the tiff specification. This is the base
#[derive(Debug)]
pub struct IFDEntry {
    pub tag: Tag,
    pub value_type: IFDValueType,
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
            let value_type = IFDValueType::from_int(value_type_raw);

            // Count
            let count = self.reader.read_u32::<T>().ok()?;
            let value_offset = self.reader.read_u32::<T>().ok()?;

            let tag_value = Tag::from(tag);
            let entry = IFDEntry {
                tag: tag_value,
                value_type: value_type,
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
