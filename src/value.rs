use endian::{Endian, Long, LongLong, Short};

use reader::IFDEntry;

use std::io::{Error, ErrorKind, Read, Result, Seek, SeekFrom};
use std::iter::Iterator;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rational<T: Long> {
    pub num: T,
    pub denom: T,
}

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

    fn value_type_id(&self) -> u16 {
        match self {
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
        let bytes = TIFFValue::read_n_bytes(reader, entry, entry.count as usize)?;

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
