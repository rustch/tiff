use std::io::{Read, Result, SeekFrom, Seek, BufReader};
use std::iter::{Iterator};
use endian::*;

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
#[derive(Debug)]
pub struct IFDEntry {
    tag: u16,
    value_type: IFDType,
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
    reader: &'a mut BufReader<R>,
    first_entry: usize,
    position: usize,
    endian: Endian,
}

impl<'a, R: Read + Seek> IFDIterator<'a, R> {

    pub fn new(reader: R, endian: Endian, first_ifd_offset: usize) -> IFDIterator<'a, R> {
        IFDIterator {
            reader: BufReader::new(reader),
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
        let next = if self.position == 0 { SeekFrom::Start(self.first_entry as u64) } else { SeekFrom::Current(self.position as i64)};
        match next {
            SeekFrom::Start(0) => {
                return None;
            }
            SeekFrom::Current(0) => {
                return None;
            }
            _ => {}
        }

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

        Some(
            IFD {
                entry_count: entry_count,
                entries: vec,
                next: next as usize
            }
        )
    }
}
