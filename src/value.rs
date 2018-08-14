use endian::Long;

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
