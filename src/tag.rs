use ifd::{IFDValue, Rational, SRational};

use std::convert::From;
use std::fmt::{Display, Error, Formatter};
use std::hash::{Hash, Hasher};

use chrono;

macro_rules! tags_id_definition {
    {$(
        $name:ident | $value:expr => $desc:expr,
    )*} => {
        #[derive(Debug, Copy, Clone, PartialEq, Eq)]
        pub enum ID {
            $($name,)*
            Unknown(u16)
        }

        impl From<u16> for ID {
            fn from(value: u16) -> ID {
                match value {
                    $( $value => ID::$name,)*
                    _ => ID::Unknown(value)
                }
        }
      }

      impl Display for ID {
          fn fmt(&self, f: &mut Formatter) -> Result<(),Error> {
              match self {
                $( ID::$name => {
                    write!(f, stringify!($name ($value): $desc))
                })*,
                ID::Unknown(value) => { write!(f, "Unkown value: {}", value) }
              }
          }
      }

          impl Hash for ID {
          fn hash<H: Hasher>(&self, state: &mut H) {
              match self {
                  $( ID::$name => {
                      $value.hash(state);
                  })*
                  ID::Unknown(val) => {
                      0xFFFF.hash(state);
                      val.hash(state);

                  }
              }
          }
    }
    }
}

tags_id_definition! {
    NewSubfileType | 0xfe	=> "A general indication of the kind of data contained in this subfile.",
    SubfileType | 0xff	=> "A general indication of the kind of data contained in this subfile.",
    ImageWidth | 0x0100	=> "The number of columns in the image, i.e., the number of pixels per row.",
    ImageLength | 0x0101	=> "The number of rows of pixels in the image.",
    BitsPerSample | 0x0102	=> "Number of bits per component.",
    Compression | 0x0103	=> "Compression scheme used on the image data.",
    PhotometricInterpretation | 0x0106	=> "The color space of the image data.",
    Threshholding | 0x0107	=> "For black and white TIFF files that represent shades of gray, the technique used to convert from gray to black and white pixels.",
    CellWidth | 0x0108	=> "The width of the dithering or halftoning matrix used to create a dithered or halftoned bilevel file.",
    CellLength | 0x0109	=> "The length of the dithering or halftoning matrix used to create a dithered or halftoned bilevel file.",
    FillOrder | 0x010a	=> "The logical order of bits within a byte.",
    ImageDescription | 0x010e	=> "A string that describes the subject of the image.",
    Make | 0x010f	=> "The scanner manufacturer.",
    Model | 0x0110	=> "The scanner model name or number.",
    StripOffsets | 0x0111	=> "For each strip, the byte offset of that strip.",
    Orientation | 0x0112	=> "The orientation of the image with respect to the rows and columns.",
    SamplesPerPixel | 0x0115	=> "The number of components per pixel.",
    RowsPerStrip | 0x0116	=> "The number of rows per strip.",
    StripByteCounts | 0x0117	=> "For each strip, the number of bytes in the strip after compression.",
    MinSampleValue | 0x0118	=> "The minimum component value used.",
    MaxSampleValue | 0x0119	=> "The maximum component value used.",
    XResolution | 0x011a	=> "The number of pixels per ResolutionUnit in the ImageWidth direction.",
    YResolution | 0x011b	=> "The number of pixels per ResolutionUnit in the ImageLength direction.",
    PlanarConfiguration | 0x011c	=> "How the components of each pixel are stored.",
    FreeOffsets | 0x0120	=> "For each string of contiguous unused bytes in a TIFF file, the byte offset of the string.",
    FreeByteCounts | 0x0121	=> "For each string of contiguous unused bytes in a TIFF file, the number of bytes in the string.",
    GrayResponseUnit | 0x0122	=> "The precision of the information contained in the GrayResponseCurve.",
    GrayResponseCurve | 0x0123	=> "For grayscale data, the optical density of each possible pixel value.",
    ResolutionUnit | 0x0128	=> "The unit of measurement for XResolution and YResolution.",
    Software | 0x0131	=> "Name and version number of the software package(s) used to create the image.",
    DateTime | 0x0132	=> "Date and time of image creation.",
    Artist | 0x013b	=> "Person who created the image.",
    HostComputer | 0x013c	=> "The computer and/or operating system in use at the time of image creation.",
    ColorMap | 0x0140	=> "A color map for palette color images.",
    ExtraSamples | 0x0152	=> "Description of extra components.",
    Copyright | 0x8298 => "Copyright notice.",
    Predictor | 0x13d => "This section defines a Predictor that greatly improves compression ratios for some images.",
}

pub trait TIFFTag: Sized {
    /// The `Tag` corresponding to this value
    fn tag() -> ID;
    /// A function creating `Self` from one `IFDValue`
    fn new_from_value(value: &IFDValue) -> Option<Self>;
}

macro_rules! short_long_value {
    ($(#[$attr:meta])* $type:ident, $tag:expr) => {
      $(#[$attr])*
        #[derive(Debug)]
        pub struct $type(pub u32);

        impl TIFFTag for $type {
            fn tag() -> ID {
                $tag
            }

            fn new_from_value(value: &IFDValue) -> Option<$type> {
                match value {
                    IFDValue::Short(el) => Some($type(el[0] as u32)),
                    IFDValue::Long(el) => Some($type(el[0])),
                    _ => None,
                }
            }
        }
    };
}

macro_rules! short_value {
    ($(#[$attr:meta])* $type:ident, $tag:expr) => {
         $(#[$attr])*
        #[derive(Debug)]
        pub struct $type(pub u16);

        impl TIFFTag for $type {
            fn tag() -> ID {
                $tag
            }

            fn new_from_value(value: &IFDValue) -> Option<$type> {
                match value {
                    IFDValue::Short(el) => Some($type(el[0] as u16)),
                    _ => None,
                }
            }
        }
    };
}

macro_rules! long_value {
    ($(#[$attr:meta])* $type:ident, $tag:expr) => {
         $(#[$attr])*
        #[derive(Debug)]
        pub struct $type(pub u16);

        impl TIFFTag for $type {
            fn tag() -> ID {
                $tag
            }

            fn new_from_value(value: &IFDValue) -> Option<$type> {
                match value {
                    IFDValue::Long(el) => Some($type(el[0] as u16)),
                    _ => None,
                }
            }
        }
    };
}

/// This Field indicates the color space of the image.
#[derive(Debug, PartialEq, Eq)]
pub enum PhotometricInterpretation {
    WhiteIsZero,
    BlackIsZero,
    RGB,
    PaletteColor,
    TransparencyMask,
    CMYK,
    YCbCr,
}

impl TIFFTag for PhotometricInterpretation {
    fn tag() -> ID {
        ID::PhotometricInterpretation
    }

    fn new_from_value(value: &IFDValue) -> Option<PhotometricInterpretation> {
        match value {
            IFDValue::Short(el) if el[0] == 0 => Some(PhotometricInterpretation::WhiteIsZero),
            IFDValue::Short(el) if el[0] == 1 => Some(PhotometricInterpretation::BlackIsZero),
            IFDValue::Short(el) if el[0] == 2 => Some(PhotometricInterpretation::RGB),
            IFDValue::Short(el) if el[0] == 3 => Some(PhotometricInterpretation::PaletteColor),
            IFDValue::Short(el) if el[0] == 4 => Some(PhotometricInterpretation::TransparencyMask),
            IFDValue::Short(el) if el[0] == 5 => Some(PhotometricInterpretation::CMYK),
            IFDValue::Short(el) if el[0] == 6 => Some(PhotometricInterpretation::YCbCr),
            _ => None,
        }
    }
}

short_long_value! {
    #[doc = "The number of columns in the image, i.e., the number of pixels per row."]
    ImageWidth,
    ID::ImageWidth
}

short_long_value!{
    #[doc = "The number of rows of pixels in the image."]
    ImageLength,
    ID::ImageLength
}

/// The unit of measurement for XResolution and YResolution
#[derive(Debug, Eq, PartialEq)]
pub enum ResolutionUnit {
    None,
    Inch,
    Centimeter,
}

impl Default for ResolutionUnit {
    fn default() -> ResolutionUnit {
        ResolutionUnit::Centimeter
    }
}

impl TIFFTag for ResolutionUnit {
    fn tag() -> ID {
        ID::ResolutionUnit
    }

    fn new_from_value(value: &IFDValue) -> Option<ResolutionUnit> {
        match value {
            IFDValue::Short(el) if el[0] == 1 => Some(ResolutionUnit::None),
            IFDValue::Short(el) if el[0] == 2 => Some(ResolutionUnit::Inch),
            IFDValue::Short(el) if el[0] == 3 => Some(ResolutionUnit::Centimeter),
            _ => None,
        }
    }
}

/// For each strip, the byte offset of that strip.
#[derive(Debug, Eq, PartialEq)]
pub struct StripOffsets(pub Vec<u32>);

impl TIFFTag for StripOffsets {
    fn tag() -> ID {
        ID::StripOffsets
    }

    fn new_from_value(value: &IFDValue) -> Option<StripOffsets> {
        match value {
            IFDValue::Short(el) => Some(StripOffsets(el.iter().map(|e| *e as u32).collect())),
            IFDValue::Long(el) => Some(StripOffsets(el.clone())),
            _ => None,
        }
    }
}

/// For each strip, the number of bytes in the strip after compression.
#[derive(Debug, Eq, PartialEq)]
pub struct StripByteCounts(pub Vec<u32>);

impl TIFFTag for StripByteCounts {
    fn tag() -> ID {
        ID::StripByteCounts
    }
    fn new_from_value(value: &IFDValue) -> Option<StripByteCounts> {
        match value {
            IFDValue::Short(el) => Some(StripByteCounts(el.iter().map(|e| *e as u32).collect())),
            IFDValue::Long(el) => Some(StripByteCounts(el.clone())),
            _ => None,
        }
    }
}

short_value!{
    #[doc = "The number of components per pixel. This number is 3 for RGB images, unless extra samples are present. See the ExtraSamples field for further information."]
    SamplesPerPixel,
    ID::SamplesPerPixel
}

impl Default for SamplesPerPixel {
    fn default() -> SamplesPerPixel {
        SamplesPerPixel(1)
    }
}

short_long_value! {
    #[doc = "The number of rows per strip."]
    RowsPerStrip,
    ID::RowsPerStrip
}

/// How the components of each pixel are stored.
#[derive(Debug, Eq, PartialEq)]
pub enum PlanarConfiguration {
    Chunky,
    Planar,
}

impl TIFFTag for PlanarConfiguration {
    fn tag() -> ID {
        ID::PlanarConfiguration
    }

    fn new_from_value(value: &IFDValue) -> Option<PlanarConfiguration> {
        match value {
            IFDValue::Short(el) if el[0] == 1 => Some(PlanarConfiguration::Chunky),
            IFDValue::Short(el) if el[0] == 2 => Some(PlanarConfiguration::Planar),
            _ => None,
        }
    }
}

/// Number of bits per component.
#[derive(Debug, Eq, PartialEq)]
pub struct BitsPerSample(pub Vec<u16>);

impl TIFFTag for BitsPerSample {
    fn tag() -> ID {
        ID::BitsPerSample
    }

    fn new_from_value(value: &IFDValue) -> Option<BitsPerSample> {
        match value {
            IFDValue::Short(el) => Some(BitsPerSample(el.clone())),
            _ => None,
        }
    }
}

/// The number of pixels per ResolutionUnit in the ImageWidth direction.
pub struct XResolution(pub Rational);

impl TIFFTag for XResolution {
    fn tag() -> ID {
        ID::XResolution
    }
    fn new_from_value(value: &IFDValue) -> Option<XResolution> {
        match value {
            IFDValue::Rational(val) => Some(XResolution(val[0])),
            _ => None,
        }
    }
}

/// The number of pixels per ResolutionUnit in the ImageLength direction.
pub struct YResolution(pub Rational);
impl TIFFTag for YResolution {
    fn tag() -> ID {
        ID::YResolution
    }

    fn new_from_value(value: &IFDValue) -> Option<YResolution> {
        match value {
            IFDValue::Rational(val) => Some(YResolution(val[0])),
            _ => None,
        }
    }
}

/// A predictor is a mathematical operator that is applied to the image data before an encoding scheme is applied.
#[derive(Debug, Eq, PartialEq)]
pub enum Predictor {
    None,
    HorizontalDifferencing,
}

impl TIFFTag for Predictor {
    fn tag() -> ID {
        ID::Predictor
    }

    fn new_from_value(value: &IFDValue) -> Option<Predictor> {
        match value {
            IFDValue::Short(el) if el[0] == 1 => Some(Predictor::None),
            IFDValue::Short(el) if el[0] == 2 => Some(Predictor::HorizontalDifferencing),
            _ => None,
        }
    }
}

/// DEPRECATED: See `NewSubfileType`
#[derive(Debug, Eq, PartialEq)]
pub enum SubfileType {
    FullResolutionImage,
    ReducedResolutionImage,
    SinglePageImage,
}

impl TIFFTag for SubfileType {
    fn tag() -> ID {
        ID::SubfileType
    }

    fn new_from_value(value: &IFDValue) -> Option<SubfileType> {
        match value {
            IFDValue::Short(el) if el[0] == 1 => Some(SubfileType::FullResolutionImage),
            IFDValue::Short(el) if el[0] == 2 => Some(SubfileType::ReducedResolutionImage),
            IFDValue::Short(el) if el[3] == 3 => Some(SubfileType::SinglePageImage),
            _ => None,
        }
    }
}

long_value! {
    #[doc = "Replaces the old SubfileType field, due to limitations in the definition of that field."]
    NewSubfileType,
    ID::NewSubfileType
}

impl NewSubfileType {
    pub fn is_reduced_image(&self) -> bool {
        0x1 & self.0 > 0
    }

    pub fn is_single_page_image(&self) -> bool {
        0x2 & self.0 > 0
    }

    pub fn is_transparency_mask_defined(&self) -> bool {
        0x4 & self.0 > 0
    }
}

/// Data can be stored either compressed or uncompressed.
pub enum Compression {
    NoCompression,
    ModifiedHuffmanCompression,
    PackBits,
}

impl TIFFTag for Compression {
    fn tag() -> ID {
        ID::Compression
    }

    fn new_from_value(value: &IFDValue) -> Option<Compression> {
        match value {
            IFDValue::Short(val) if val[0] == 1 => Some(Compression::NoCompression),
            IFDValue::Short(val) if val[0] == 2 => Some(Compression::ModifiedHuffmanCompression),
            IFDValue::Short(val) if val[0] == 32773 => Some(Compression::PackBits),
            _ => None,
        }
    }
}

/// Name and version number of the software package(s) used to create the image.
pub struct Software(pub String);

impl TIFFTag for Software {
    fn tag() -> ID {
        ID::Software
    }

    fn new_from_value(value: &IFDValue) -> Option<Software> {
        match value {
            IFDValue::Ascii(val) => Some(Software(val[0].clone())),
            _ => None,
        }
    }
}

pub struct DateTime(pub chrono::DateTime<chrono::FixedOffset>);

impl TIFFTag for DateTime {
    fn tag() -> ID {
        ID::DateTime
    }

    fn new_from_value(value: &IFDValue) -> Option<DateTime> {
        match value {
            IFDValue::Ascii(val) => {
                let time = chrono::DateTime::parse_from_str(&val[0], "%Y:%m:%d %H:%M:%S").ok()?;
                Some(DateTime(time))
            }
            _ => None,
        }
    }
}
