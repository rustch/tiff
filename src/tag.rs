use ifd::IFDValue;

use std::convert::From;
use std::fmt::{Display, Error, Formatter};
use std::hash::{Hash, Hasher};

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
    NewSubfileType | 0x00fe	=> "A general indication of the kind of data contained in this subfile.",
    SubfileType | 0x00ff	=> "A general indication of the kind of data contained in this subfile.",
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
    ($type:ident, $tag:expr) => {
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
    ($type:ident, $tag:expr) => {
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

/// The value of the `PhotometricInterpretation` value from the
/// TIFF specification.
#[derive(Debug, PartialEq, Eq)]
pub enum PhotometricInterpretation {
    WhiteIsZero,
    BlackIsZero,
    RGB,
    PaletteColor,
    TransparencyMask,
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
            _ => None,
        }
    }
}

short_long_value!(ImageWidth, ID::ImageWidth);

short_long_value!(ImageLength, ID::ImageLength);

#[derive(Debug)]
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

short_long_value!(StripOffsets, ID::StripOffsets);

short_value!(SamplesPerPixel, ID::SamplesPerPixel);

impl Default for SamplesPerPixel {
    fn default() -> SamplesPerPixel {
        SamplesPerPixel(1)
    }
}

short_long_value!(RowsPerStrip, ID::RowsPerStrip);

short_long_value!(StripByteCounts, ID::StripByteCounts);

#[derive(Debug)]
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

#[derive(Debug)]
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
