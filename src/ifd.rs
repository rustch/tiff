use endian::*;
use std::io::{BufReader, Read, Result, Seek, SeekFrom};
use std::iter::Iterator;

enum IFDValue {
    Byte(u8),
    Ascii([u8; 8]),
    Short(u16),
    Long(u32),
    Rational{ num: u32, denum: u32 },
    SByte(i8),
    Undefined(u8),
    SShort(i16),
    SLong(i32),
    SRational{ num: i32, denum: i32 },
    Float(f32),
    Double(f64),
}

#[derive(Debug)]
enum IFDType {
    Byte,
    Ascii,
    Short,
    Long,
    Rational,
    SByte,
    Undefined(u16),
    SShort,
    SLong,
    SRational,
    Float,
    Double,
}

impl IFDType {
    fn from_int(value: u16) -> IFDType {
        match value {
            1 => IFDType::Byte,
            2 => IFDType::Ascii,
            3 => IFDType::Short,
            4 => IFDType::Long,
            5 => IFDType::Rational,
            6 => IFDType::SByte,
            7 => IFDType::Undefined(7),
            8 => IFDType::SShort,
            9 => IFDType::SLong,
            10 => IFDType::SRational,
            11 => IFDType::Float,
            12 => IFDType::Double,
            _ => IFDType::Undefined(value),
        }
    }
}

/// An `IFDEntry` represents an **image file directory**
/// mentionned inside the tiff specification. This is the base
/// l
#[derive(Debug)]
pub struct IFDEntry {
    tag: u16,
    value_type: IFDType,
    count: u32,
    value_offset: u32,
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

impl<'a, R: 'a + Read + Seek> IFDIterator<'a, R> {
    pub fn new(reader: &'a mut R, endian: Endian, first_ifd_offset: usize) -> IFDIterator<'a, R> {
        
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

impl<'a, R: Read + Seek> Iterator for IFDIterator<'a, R> where R: 'a {
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
            let value_type = IFDType::from_int(value_type_raw);

            // Count
            let count = self.read_u32().ok()?;

            // Value Offset
            let value_offset = self.read_u32().ok()?;

            let entry = IFDEntry {
                tag: tag,
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