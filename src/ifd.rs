use endian::*;
use std::io::{Read, Result, Seek, SeekFrom};
use std::iter::Iterator;
use tag::Tag;
use std::borrow::Cow;
use std::convert::From;

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
    tag: Tag,
    value_type: IFDValueType,
    count: u32,
    value_offset: u32
}

#[derive(Debug)]
pub struct IFD {
    entry_count: u16,
    entries: Vec<IFDEntry>,
    next: usize,
}

pub struct IFDIterator<'a, R: 'a> {
    reader: &'a mut R,
    first_entry: usize,
    position: usize,
    endian: Endian,
}

impl<'a, R: Read + Seek> IFDIterator<'a,  R> where R: 'a {
    pub fn new(reader: &'a mut R, endian: Endian, first_ifd_offset: usize) -> IFDIterator<R> {
        
        reader.seek(SeekFrom::Start(0));

        IFDIterator {
            reader: reader,
            first_entry: first_ifd_offset,
            endian: endian,
            position: 0,
        }
    }

    fn read_u16(&mut self) -> Result<u16> {
        let mut buf: [u8; 2] = [0; 2];
        self.reader.read_exact(&mut buf)?;
        self.position += 2;
        Ok(read_u16_from_endian(&self.endian, buf))
    }

    fn read_u32(&mut self) -> Result<u32> {
        let mut buf: [u8; 4] = [0; 4];
        self.reader.read_exact(&mut buf)?;
        self.position += 4;
        Ok(read_u32_from_endian(&self.endian, buf))
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
        let entry_count = self.read_u16().ok()?;
        let mut vec = Vec::<IFDEntry>::with_capacity(entry_count as usize);
        for _i in 0..entry_count {
            // Tag
            let tag = self.read_u16().ok()?;

            // Type
            let value_type_raw = self.read_u16().ok()?;
            let value_type = IFDValueType::from_int(value_type_raw);

            // Count
            let count = self.read_u32().ok()?;
            let value_offset = self.read_u32().ok()?;

            let entry = IFDEntry {
                tag: Tag::from(tag),
                value_type: value_type,
                count: count,
                value_offset: value_offset,
            };

            vec.push(entry);
        }

        let next = self.read_u32().ok()?;

        Some(IFD {
            entry_count: entry_count,
            entries: vec,
            next: next as usize,
        })
    }
}