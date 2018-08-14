use value::{Rational, TIFFValue};

use chrono;
use std::convert::From;
use std::fmt::{Display, Error, Formatter};
use std::hash::{Hash, Hasher};

macro_rules! tags_id_definition {
    {$(
        $name:ident | $value:expr => $desc:expr,
    )*} => {
        #[derive(Debug, Copy, Clone, PartialEq, Eq)]
        pub enum Tag {
            $($name,)*
            Unknown(u16)
        }

        impl From<u16> for Tag {
            fn from(value: u16) -> Tag {
                match value {
                    $( $value => Tag::$name,)*
                    _ => Tag::Unknown(value)
                }
        }
      }

      impl Display for Tag {
          fn fmt(&self, f: &mut Formatter) -> Result<(),Error> {
              match self {
                $( Tag::$name => {
                    write!(f, stringify!($name ($value): $desc))
                })*,
                Tag::Unknown(value) => { write!(f, "Unkown value: {}", value) }
              }
          }
      }

          impl Hash for Tag {
          fn hash<H: Hasher>(&self, state: &mut H) {
              match self {
                  $( Tag::$name => {
                      $value.hash(state);
                  })*
                  Tag::Unknown(val) => {
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
    T4Options | 0x124 => "See Compression=3. This field is made up of a set of 32 flag bits. Unused bits must be set to 0. Bit 0 is the low-order bit.",
    T6Options | 0x125 => "See Compression=3. See Compression = 4. This field is made up of a set of 32 flag bits. Unused bits must be set to 0. Bit 0 is the low-order bit. The default value is 0 (all bits 0).",
    DocumentName | 0x10D => "The name of the document from which this image was scanned.",
    PageName | 0x11D => "The name of the page from which this image was scanned.",
    PageNumber | 0x129 => "The page number of the page from which this image was scanned.",
    XPosition | 0x11E => "X position of the image.",
    YPosition | 0x11F => "Y position of the image.",
    TileWidth | 0x142 => "The tile width in pixels. This is the number of columns in each tile.",
    TileLength | 0x143 => "The tile length (height) in pixels. This is the number of rows in each tile.",
    TileOffsets | 0x144 => "For each tile, the byte offset of that tile, as compressed and stored on disk.",
    TileByteCounts | 0x145 => "For each tile, the number of (compressed) bytes in that tile.",
    InkSet | 0x14c => "The set of inks used in a separated (PhotometricInterpretation=5) image.",
    NumberOfInks | 0x14e => "The number of inks. Usually equal to SamplesPerPixel, unless there are extra samples.",
    InkNames | 0x14d => "The name of each ink used in a separated (PhotometricInterpretation=5) image, written as a list of concatenated, NUL-terminated ASCII strings. The number of strings must be equal to NumberOfInks.",
    DotRange | 0x150 => "The component values that correspond to a 0% dot and 100% dot. DotRange[0] corresponds to a 0% dot, and DotRange[1] corresponds to a 100% dot.",
    TargetPrinter | 0x151 => "A description of the printing environment for which this separation is intended.",
    HalftoneHints | 0x141 => "The purpose of the HalftoneHints field is to convey to the halftone function the range of gray levels within a colorimetrically-specified image that should retain tonal detail.",
    SampleFormat | 0x153 => "This field specifies how to interpret each data sample in a pixel. Possible values are:",
    SMinSampleValue | 0x154 => "This field specifies the minimum sample value. Note that a value should be given for each data sample. That is, if the image has 3 SamplesPerPixel, 3 values must be specified.",
    SMaxSampleValue | 0x155 => "This new field specifies the maximum sample value.",
    WhitePoint | 0x318 => "The chromaticity of the white point of the image.",
    PrimaryChromaticities | 0x319 => "The chromaticities of the primaries of the image.",
    TransferFunction | 0x301 => "Describes a transfer function for the image in tabular style.",
    TransferRange | 0x156 => "Expands the range of the TransferFunction",
    ReferenceBlackWhite | 0x532 => "Specifies a pair of headroom and footroom image data values (codes) for each pixel component",
    YCbCrCoefficients | 0x211 => "The transformation from RGB to YC C image data",
    YCbCrSubSampling | 0x212 => "Specifies the subsampling factors used for the chrominance components of a YC C image.",
    YCbCrPositioning | 0x213 => "Specifies the positioning of subsampled chrominance components relative to luminance samples.",
    JPEGProc | 0x200 => "This Field indicates the JPEG process used to produce the compressed data.",
    JPEGInterchangeFormat | 0x201 => "This Field indicates whether a JPEG interchange format bitstream is present in the TIFF file",
    JPEGInterchangeFormatLength | 0x202 => "This Field indicates the length in bytes of the JPEG interchange format bitstream.",
    JPEGRestartInterval | 0x203 => "This Field indicates the length of the restart interval used in the compressed image data.",
    JPEGLosslessPredictors | 0x205 => "This Field points to a list of lossless predictor-selection values, one per compo- nent.",
    JPEGPointTransforms | 0x206 => "This Field points to a list of point transform values, one per component. This Field is relevant only for lossless processes.",
    JPEGQTables | 0x207 => "This Field points to a list of offsets to the quantization tables, one per component.",
    JPEGDCTables | 0x208 => "This Field points to a list of offsets to the DC Huffman tables or the lossless Huffman tables, one per component",
    JPEGACTables | 0x209 => "This Field points to a list of offsets to the Huffman AC tables, one per component.",
}

pub trait Field: Sized {
    /// The `Tag` corresponding to this value
    fn tag() -> Tag;

    /// A function creating `Self` from one `TIFFValue`
    fn decode_from_value(value: &TIFFValue) -> Option<Self>;

    fn encode_to_value(&self) -> Option<TIFFValue>;
}

macro_rules! ascii_value {
    ($(#[$attr:meta])* $type:ident, $tag:expr) => {
      $(#[$attr])*
        #[derive(Debug)]
        pub struct $type(pub String);

        impl Field for $type {
            fn tag() -> Tag {
                $tag
            }

            fn decode_from_value(value: &TIFFValue) -> Option<$type> {
                match value {
                    TIFFValue::Ascii(el) => Some($type(el[0].clone())),
                    _ => None,
                }
            }

            fn encode_to_value(&self) -> Option<TIFFValue> {
                Some(TIFFValue::Ascii(vec![self.0.clone()]))
            }
        }
    };
}

macro_rules! short_long_value {
    ($(#[$attr:meta])* $type:ident, $tag:expr) => {
      $(#[$attr])*
        #[derive(Debug)]
        pub struct $type(pub u32);

        impl Field for $type {
            fn tag() -> Tag {
                $tag
            }

            fn decode_from_value(value: &TIFFValue) -> Option<$type> {
                match value {
                    TIFFValue::Short(el) => Some($type(el[0] as u32)),
                    TIFFValue::Long(el) => Some($type(el[0])),
                    _ => None,
                }
            }

            fn encode_to_value(&self) -> Option<TIFFValue> {
                if self.0 <= ::std::u16::MAX as u32 {
                    Some(TIFFValue::Short(vec![self.0 as u16]))
                } else {
                    Some(TIFFValue::Long(vec![self.0]))
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

        impl Field for $type {
            fn tag() -> Tag {
                $tag
            }

            fn decode_from_value(value: &TIFFValue) -> Option<$type> {
                match value {
                    TIFFValue::Short(el) => Some($type(el[0] as u16)),
                    _ => None,
                }
            }

             fn encode_to_value(&self) -> Option<TIFFValue> {
                 Some(TIFFValue::Short(vec![self.0]))
             }
        }
    };
}

macro_rules! long_value {
    ($(#[$attr:meta])* $type:ident, $tag:expr) => {
         $(#[$attr])*
        #[derive(Debug)]
        pub struct $type(pub u32);

        impl Field for $type {
            fn tag() -> Tag {
                $tag
            }

            fn decode_from_value(value: &TIFFValue) -> Option<$type> {
                match value {
                    TIFFValue::Long(el) => Some($type(el[0])),
                    _ => None,
                }
            }

             fn encode_to_value(&self) -> Option<TIFFValue> {
                 Some(TIFFValue::Long(vec![self.0]))
             }
        }
    };
}

macro_rules! vec_short_u_value {
    ($(#[$attr:meta])* $type:ident, $tag:expr) => {
         $(#[$attr])*
        #[derive(Debug)]
        pub struct $type(pub Vec<u16>);

        impl Field for $type {
            fn tag() -> Tag {
                $tag
            }

            fn decode_from_value(value: &TIFFValue) -> Option<$type> {
                match value {
                    TIFFValue::Short(el) => Some($type(el.clone())),
                    _ => None,
                }
            }

            fn encode_to_value(&self) -> Option<TIFFValue> {
                 Some(TIFFValue::Short(self.0.clone()))
            }
        }
    };
}
macro_rules! rational_value {
    ($(#[$attr:meta])* $type:ident, $tag:expr) => {
         $(#[$attr])*
        #[derive(Debug)]
        pub struct $type(pub Rational<u32>);

        impl Field for $type {
            fn tag() -> Tag {
                $tag
            }

            fn decode_from_value(value: &TIFFValue) -> Option<$type> {
                match value {
                    TIFFValue::Rational(el) => Some($type(el[0])),
                    _ => None,
                }
            }

            fn encode_to_value(&self) -> Option<TIFFValue> {
                 Some(TIFFValue::Rational(vec![self.0]))
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

impl Field for PhotometricInterpretation {
    fn tag() -> Tag {
        Tag::PhotometricInterpretation
    }

    fn decode_from_value(value: &TIFFValue) -> Option<PhotometricInterpretation> {
        match value {
            TIFFValue::Short(el) if el[0] == 0 => Some(PhotometricInterpretation::WhiteIsZero),
            TIFFValue::Short(el) if el[0] == 1 => Some(PhotometricInterpretation::BlackIsZero),
            TIFFValue::Short(el) if el[0] == 2 => Some(PhotometricInterpretation::RGB),
            TIFFValue::Short(el) if el[0] == 3 => Some(PhotometricInterpretation::PaletteColor),
            TIFFValue::Short(el) if el[0] == 4 => Some(PhotometricInterpretation::TransparencyMask),
            TIFFValue::Short(el) if el[0] == 5 => Some(PhotometricInterpretation::CMYK),
            TIFFValue::Short(el) if el[0] == 6 => Some(PhotometricInterpretation::YCbCr),
            _ => None,
        }
    }

    fn encode_to_value(&self) -> Option<TIFFValue> {
        let short_value: u16 = match self {
            PhotometricInterpretation::WhiteIsZero => 0,
            PhotometricInterpretation::BlackIsZero => 1,
            PhotometricInterpretation::RGB => 2,
            PhotometricInterpretation::PaletteColor => 3,
            PhotometricInterpretation::TransparencyMask => 4,
            PhotometricInterpretation::CMYK => 5,
            PhotometricInterpretation::YCbCr => 6,
        };

        Some(TIFFValue::Short(vec![short_value]))
    }
}

short_long_value! {
    #[doc = "The number of columns in the image, i.e., the number of pixels per row."]
    ImageWidth,
    Tag::ImageWidth
}

short_long_value!{
    #[doc = "The number of rows of pixels in the image."]
    ImageLength,
    Tag::ImageLength
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

impl Field for ResolutionUnit {
    fn tag() -> Tag {
        Tag::ResolutionUnit
    }

    fn decode_from_value(value: &TIFFValue) -> Option<ResolutionUnit> {
        match value {
            TIFFValue::Short(el) if el[0] == 1 => Some(ResolutionUnit::None),
            TIFFValue::Short(el) if el[0] == 2 => Some(ResolutionUnit::Inch),
            TIFFValue::Short(el) if el[0] == 3 => Some(ResolutionUnit::Centimeter),
            _ => None,
        }
    }

    fn encode_to_value(&self) -> Option<TIFFValue> {
        let raw: u16 = match self {
            ResolutionUnit::None => 1,
            ResolutionUnit::Inch => 2,
            ResolutionUnit::Centimeter => 3,
        };

        Some(TIFFValue::Short(vec![raw]))
    }
}

/// For each strip, the byte offset of that strip.
#[derive(Debug, Eq, PartialEq)]
pub struct StripOffsets(pub Vec<u32>);

impl Field for StripOffsets {
    fn tag() -> Tag {
        Tag::StripOffsets
    }

    fn decode_from_value(value: &TIFFValue) -> Option<StripOffsets> {
        match value {
            TIFFValue::Short(el) => Some(StripOffsets(el.iter().map(|e| *e as u32).collect())),
            TIFFValue::Long(el) => Some(StripOffsets(el.clone())),
            _ => None,
        }
    }

    fn encode_to_value(&self) -> Option<TIFFValue> {
        let is_big = self
            .0
            .iter()
            .filter(|x| **x > (::std::u16::MAX as u32))
            .collect::<Vec<&u32>>()
            .len()
            > 0;

        if is_big {
            Some(TIFFValue::Long(self.0.clone()))
        } else {
            let lower = self.0.iter().map(|e| *e as u16).collect();
            Some(TIFFValue::Short(lower))
        }
    }
}

/// For each strip, the number of bytes in the strip after compression.
#[derive(Debug, Eq, PartialEq)]
pub struct StripByteCounts(pub Vec<u32>);

impl Field for StripByteCounts {
    fn tag() -> Tag {
        Tag::StripByteCounts
    }
    fn decode_from_value(value: &TIFFValue) -> Option<StripByteCounts> {
        match value {
            TIFFValue::Short(el) => Some(StripByteCounts(el.iter().map(|e| *e as u32).collect())),
            TIFFValue::Long(el) => Some(StripByteCounts(el.clone())),
            _ => None,
        }
    }

    fn encode_to_value(&self) -> Option<TIFFValue> {
        let is_big = self
            .0
            .iter()
            .filter(|x| **x > (::std::u16::MAX as u32))
            .collect::<Vec<&u32>>()
            .len()
            > 0;

        if is_big {
            Some(TIFFValue::Long(self.0.clone()))
        } else {
            let lower = self.0.iter().map(|e| *e as u16).collect();
            Some(TIFFValue::Short(lower))
        }
    }
}

short_value!{
    #[doc = "The number of components per pixel. This number is 3 for RGB images, unless extra samples are present. See the ExtraSamples field for further information."]
    SamplesPerPixel,
    Tag::SamplesPerPixel
}

impl Default for SamplesPerPixel {
    fn default() -> SamplesPerPixel {
        SamplesPerPixel(1)
    }
}

short_long_value! {
    #[doc = "The number of rows per strip."]
    RowsPerStrip,
    Tag::RowsPerStrip
}

/// How the components of each pixel are stored.
#[derive(Debug, Eq, PartialEq)]
pub enum PlanarConfiguration {
    Chunky,
    Planar,
}

impl Field for PlanarConfiguration {
    fn tag() -> Tag {
        Tag::PlanarConfiguration
    }

    fn decode_from_value(value: &TIFFValue) -> Option<PlanarConfiguration> {
        match value {
            TIFFValue::Short(el) if el[0] == 1 => Some(PlanarConfiguration::Chunky),
            TIFFValue::Short(el) if el[0] == 2 => Some(PlanarConfiguration::Planar),
            _ => None,
        }
    }

    fn encode_to_value(&self) -> Option<TIFFValue> {
        let value = match self {
            PlanarConfiguration::Chunky => 1,
            PlanarConfiguration::Planar => 2,
        };

        Some(TIFFValue::Short(vec![value]))
    }
}

/// Number of bits per component.
#[derive(Debug, Eq, PartialEq)]
pub struct BitsPerSample(pub Vec<u16>);

impl Field for BitsPerSample {
    fn tag() -> Tag {
        Tag::BitsPerSample
    }

    fn decode_from_value(value: &TIFFValue) -> Option<BitsPerSample> {
        match value {
            TIFFValue::Short(el) => Some(BitsPerSample(el.clone())),
            _ => None,
        }
    }

    fn encode_to_value(&self) -> Option<TIFFValue> {
        Some(TIFFValue::Short(self.0.clone()))
    }
}

rational_value! {
    #[doc = "The number of pixels per ResolutionUnit in the ImageWidth direction."]
    XResolution,
    Tag::XResolution
}

rational_value! {
    #[doc = "The number of pixels per ResolutionUnit in the ImageLength direction."]
    YResolution,
    Tag::YResolution
}

/// A predictor is a mathematical operator that is applied to the image data before an encoding scheme is applied.
#[derive(Debug, Eq, PartialEq)]
pub enum Predictor {
    None,
    HorizontalDifferencing,
}

impl Field for Predictor {
    fn tag() -> Tag {
        Tag::Predictor
    }

    fn decode_from_value(value: &TIFFValue) -> Option<Predictor> {
        match value {
            TIFFValue::Short(el) if el[0] == 1 => Some(Predictor::None),
            TIFFValue::Short(el) if el[0] == 2 => Some(Predictor::HorizontalDifferencing),
            _ => None,
        }
    }

    fn encode_to_value(&self) -> Option<TIFFValue> {
        let value = match self {
            Predictor::None => 1,
            Predictor::HorizontalDifferencing => 2,
        };
        Some(TIFFValue::Short(vec![value]))
    }
}

/// DEPRECATED: See `NewSubfileType`
#[derive(Debug, Eq, PartialEq)]
pub enum SubfileType {
    FullResolutionImage,
    ReducedResolutionImage,
    SinglePageImage,
}

impl Field for SubfileType {
    fn tag() -> Tag {
        Tag::SubfileType
    }

    fn decode_from_value(value: &TIFFValue) -> Option<SubfileType> {
        match value {
            TIFFValue::Short(el) if el[0] == 1 => Some(SubfileType::FullResolutionImage),
            TIFFValue::Short(el) if el[0] == 2 => Some(SubfileType::ReducedResolutionImage),
            TIFFValue::Short(el) if el[3] == 3 => Some(SubfileType::SinglePageImage),
            _ => None,
        }
    }

    fn encode_to_value(&self) -> Option<TIFFValue> {
        let value = match self {
            SubfileType::FullResolutionImage => 1,
            SubfileType::ReducedResolutionImage => 2,
            SubfileType::SinglePageImage => 3,
        };
        Some(TIFFValue::Short(vec![value]))
    }
}

long_value! {
    #[doc = "Replaces the old SubfileType field, due to limitations in the definition of that field."]
    NewSubfileType,
    Tag::NewSubfileType
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

impl Field for Compression {
    fn tag() -> Tag {
        Tag::Compression
    }

    fn decode_from_value(value: &TIFFValue) -> Option<Compression> {
        match value {
            TIFFValue::Short(val) if val[0] == 1 => Some(Compression::NoCompression),
            TIFFValue::Short(val) if val[0] == 2 => Some(Compression::ModifiedHuffmanCompression),
            TIFFValue::Short(val) if val[0] == 32773 => Some(Compression::PackBits),
            _ => None,
        }
    }

    fn encode_to_value(&self) -> Option<TIFFValue> {
        let value = match self {
            Compression::NoCompression => 1,
            Compression::ModifiedHuffmanCompression => 2,
            Compression::PackBits => 32773,
        };

        Some(TIFFValue::Short(vec![value]))
    }
}

ascii_value! {
    #[doc = "Name and version number of the software package(s) used to create the image."]
    Software,
    Tag::Software
}

pub struct DateTime(pub chrono::DateTime<chrono::FixedOffset>);

impl Field for DateTime {
    fn tag() -> Tag {
        Tag::DateTime
    }

    fn decode_from_value(value: &TIFFValue) -> Option<DateTime> {
        match value {
            TIFFValue::Ascii(val) => {
                let time = chrono::DateTime::parse_from_str(&val[0], "%Y:%m:%d %H:%M:%S").ok()?;
                Some(DateTime(time))
            }
            _ => None,
        }
    }

    fn encode_to_value(&self) -> Option<TIFFValue> {
        Some(TIFFValue::Ascii(vec![self.0.to_string()]))
    }
}

short_value!{
    #[doc = "The length of the dithering or halftoning matrix used to create a dithered or halftoned bilevel file."]
    CellLength,
    Tag::CellLength
}

short_value!{
    #[doc = "The width of the dithering or halftoning matrix used to create a dithered or halftoned bilevel file;"]
    CellWidth,
    Tag::CellWidth
}

/// The color map for colored images
///
/// This field defines a Red-Green-Blue color map (often called a lookup table) for palette color images.
/// In a palette-color image, a pixel value is used to index into an RGB-lookup table.
/// For example, a palette-color pixel having a value of 0 would be displayed
/// according to the 0th Red, Green, Blue triplet.
/// In a TIFF ColorMap, all the Red values come first, followed by the Green values, then the Blue values.
/// In the ColorMap, black is represented by 0,0,0 and white is represented by 65535, 65535, 65535.
pub struct ColorMap(Vec<u16>);

impl Field for ColorMap {
    fn tag() -> Tag {
        Tag::ColorMap
    }

    fn decode_from_value(value: &TIFFValue) -> Option<ColorMap> {
        match value {
            TIFFValue::Short(e) => Some(ColorMap(e.clone())),
            _ => None,
        }
    }

    fn encode_to_value(&self) -> Option<TIFFValue> {
        Some(TIFFValue::Short(self.0.clone()))
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ExtraSampleDataValue {
    Unspecified,
    AssociatedAlpha,
    UnassociatedAlpha,
}

impl ExtraSampleDataValue {
    fn from_value(value: u16) -> ExtraSampleDataValue {
        match value {
            0 => ExtraSampleDataValue::Unspecified,
            1 => ExtraSampleDataValue::AssociatedAlpha,
            2 => ExtraSampleDataValue::UnassociatedAlpha,
            _ => panic!("Invalid ExtraSampleDataValue"),
        }
    }

    fn to_value(&self) -> u16 {
        match self {
            ExtraSampleDataValue::Unspecified => 0,
            ExtraSampleDataValue::AssociatedAlpha => 1,
            ExtraSampleDataValue::UnassociatedAlpha => 3,
        }
    }
}

ascii_value! {
      #[doc = "Copyright notice."]
      Copyright,
      Tag::Copyright
}
/// Description of extra components.
///
/// Specifies that each pixel has m extra components whose interpretation is defined by one of the values l
/// isted below. When this field is used, the SamplesPerPixel field has a value greater than the
/// PhotometricInterpretation field suggests.
/// For example, full-color RGB data normally has SamplesPerPixel=3.
/// If SamplesPerPixel is greater than 3, then the ExtraSamples field describes the meaning of the extra samples.
/// If SamplesPerPixel is, say, 5 then ExtraSamples will contain 2 values, one for each extra sample.
struct ExtraSamples(pub Vec<ExtraSampleDataValue>);

impl Field for ExtraSamples {
    fn tag() -> Tag {
        Tag::ExtraSamples
    }

    fn decode_from_value(value: &TIFFValue) -> Option<ExtraSamples> {
        let raw = match value {
            TIFFValue::Short(e) => e,
            _ => return None,
        };

        let values: Vec<ExtraSampleDataValue> = raw
            .iter()
            .map(|e| ExtraSampleDataValue::from_value(*e))
            .collect();
        Some(ExtraSamples(values))
    }

    fn encode_to_value(&self) -> Option<TIFFValue> {
        let values = self.0.iter().map(|e| e.to_value()).collect();
        Some(TIFFValue::Short(values))
    }
}

/// The logical order of bits within a byte.

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FillOrder {
    LowerColumnsToHigherOrderBits,
    LowerColumnsToLowerOrderBits,
}

impl Field for FillOrder {
    fn tag() -> Tag {
        Tag::FillOrder
    }

    fn decode_from_value(value: &TIFFValue) -> Option<FillOrder> {
        match value {
            TIFFValue::Short(e) if e[0] == 1 => Some(FillOrder::LowerColumnsToHigherOrderBits),
            TIFFValue::Short(e) if e[0] == 2 => Some(FillOrder::LowerColumnsToLowerOrderBits),
            _ => None,
        }
    }

    fn encode_to_value(&self) -> Option<TIFFValue> {
        let val = match self {
            FillOrder::LowerColumnsToHigherOrderBits => 1,
            FillOrder::LowerColumnsToLowerOrderBits => 2,
        };
        Some(TIFFValue::Short(vec![val]))
    }
}

impl Default for FillOrder {
    fn default() -> FillOrder {
        FillOrder::LowerColumnsToHigherOrderBits
    }
}

long_value! {
    #[doc = "For each string of contiguous unused bytes in a TIFF file, the number of bytes in the string."]
    FreeByteCounts,
    Tag::FreeByteCounts
}

long_value! {
    #[doc = "For each string of contiguous unused bytes in a TIFF file, the byte offset of the string."]
    FreeOffsets,
    Tag::FreeOffsets
}

vec_short_u_value! {
    #[doc = "For grayscale data, the optical density of each possible pixel value."]
    GrayResponseCurve,
    Tag::GrayResponseCurve
}

/// The precision of the information contained in the GrayResponseCurve.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum GrayResponseUnit {
    TenthsOfUnit,
    HundredthsOfUnit,
    ThousandthsOfUnit,
    TenThousandthsOfUnit,
    HundredThousandthsOfUnit,
}

impl Default for GrayResponseUnit {
    fn default() -> GrayResponseUnit {
        GrayResponseUnit::HundredthsOfUnit
    }
}
impl Field for GrayResponseUnit {
    fn tag() -> Tag {
        Tag::GrayResponseUnit
    }

    fn decode_from_value(value: &TIFFValue) -> Option<GrayResponseUnit> {
        match value {
            TIFFValue::Short(e) if e[0] == 1 => Some(GrayResponseUnit::TenthsOfUnit),
            TIFFValue::Short(e) if e[0] == 2 => Some(GrayResponseUnit::HundredthsOfUnit),
            TIFFValue::Short(e) if e[0] == 3 => Some(GrayResponseUnit::ThousandthsOfUnit),
            TIFFValue::Short(e) if e[0] == 4 => Some(GrayResponseUnit::TenThousandthsOfUnit),
            TIFFValue::Short(e) if e[0] == 5 => Some(GrayResponseUnit::HundredThousandthsOfUnit),
            _ => None,
        }
    }

    fn encode_to_value(&self) -> Option<TIFFValue> {
        let value = match self {
            GrayResponseUnit::TenthsOfUnit => 1,
            GrayResponseUnit::HundredthsOfUnit => 2,
            GrayResponseUnit::ThousandthsOfUnit => 3,
            GrayResponseUnit::TenThousandthsOfUnit => 4,
            GrayResponseUnit::HundredThousandthsOfUnit => 5,
        };

        Some(TIFFValue::Short(vec![value]))
    }
}

ascii_value! {
    #[doc = "The computer and/or operating system in use at the time of image creation."]
    HostComputer,
    Tag::HostComputer
}

ascii_value! {
    #[doc = "A string that describes the subject of the image."]
    ImageDescription,
    Tag::ImageDescription
}

ascii_value! {
    #[doc = "The scanner manufacturer."]
    Make,
    Tag::Make
}

vec_short_u_value! {
    #[doc = "The maximum component value used."]
    MaxSampleValue,
    Tag::MaxSampleValue
}

vec_short_u_value! {
    #[doc = "The minimum component value used."]
    MinSampleValue,
    Tag::MinSampleValue
}

ascii_value! {
    #[doc = "The scanner model name or number."]
    Model,
    Tag::Model
}

short_value! {
    #[doc = "For black and white TIFF files that represent shades of gray, the technique used to convert from gray to black and white pixels."]
    Threshholding,
    Tag::Threshholding
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Orientation {
    RTopCLeft,
    RTopCRight,
    RBottomCRight,
    RBottomCLeft,
    RLeftCTop,
    RRightCTop,
    RRightCBottom,
    RLeftCBottom,
}

impl Field for Orientation {
    fn tag() -> Tag {
        Tag::Orientation
    }

    fn decode_from_value(value: &TIFFValue) -> Option<Orientation> {
        let val = match value {
            TIFFValue::Short(v) => v[0],
            _ => return None,
        };

        let ret = match val {
            1 => Orientation::RTopCLeft,
            2 => Orientation::RTopCRight,
            3 => Orientation::RBottomCRight,
            4 => Orientation::RBottomCLeft,
            5 => Orientation::RLeftCTop,
            6 => Orientation::RRightCTop,
            7 => Orientation::RRightCBottom,
            8 => Orientation::RLeftCBottom,
            _ => return None,
        };

        Some(ret)
    }

    fn encode_to_value(&self) -> Option<TIFFValue> {
        let ret = match self {
            Orientation::RTopCLeft => 1,
            Orientation::RTopCRight => 2,
            Orientation::RBottomCRight => 3,
            Orientation::RBottomCLeft => 4,
            Orientation::RLeftCTop => 5,
            Orientation::RRightCTop => 6,
            Orientation::RRightCBottom => 7,
            Orientation::RLeftCBottom => 8,
        };
        Some(TIFFValue::Short(vec![ret]))
    }
}

long_value! {
    #[doc = "See Compression=3. This field is made up of a set of 32 flag bits. Unused bits must be set to 0. Bit 0 is the low-order bit."]
    T4Options,
    Tag::T4Options
}

long_value! {
    #[doc = "See Compression = 4. This field is made up of a set of 32 flag bits. Unused bits must be set to 0. Bit 0 is the low-order bit. The default value is 0 (all bits 0)."]
    T6Options,
    Tag::T6Options
}

ascii_value! {
    #[doc = "The name of the document from which this image was scanned."]
    DocumentName,
    Tag::DocumentName
}

ascii_value! {
    #[doc = "The name of the page from which this image was scanned."]
    PageName,
    Tag::PageName
}

short_value! {
    #[doc = "The page number of the page from which this image was scanned."]
    PageNumber,
    Tag::PageNumber
}

rational_value! {
    #[doc = "X position of the image."]
    XPosition,
    Tag::XPosition
}

rational_value! {
    #[doc = "Y position of the image."]
    YPosition,
    Tag::YPosition
}

short_long_value! {
    #[doc = "The tile width in pixels."]
    TileWidth,
    Tag::TileWidth
}

short_long_value! {
    #[doc = "The tile length (height) in pixels"]
    TileLength,
    Tag::TileLength
}

long_value! {
    #[doc = "For each tile, the byte offset of that tile, as compressed and stored on disk"]
    TileOffsets,
    Tag::TileOffsets
}

short_long_value! {
    #[doc = "For each tile, the number of (compressed) bytes in that tile."]
    TileByteCounts,
    Tag::TileByteCounts
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum InkSet {
    CMYK,
    NotCMYK,
}

impl Field for InkSet {
    fn tag() -> Tag {
        Tag::InkSet
    }

    fn decode_from_value(value: &TIFFValue) -> Option<InkSet> {
        let val = match value {
            TIFFValue::Short(val) => val[0],
            _ => return None,
        };

        let res = match val {
            1 => InkSet::CMYK,
            2 => InkSet::NotCMYK,
            _ => return None,
        };
        Some(res)
    }

    fn encode_to_value(&self) -> Option<TIFFValue> {
        let val = match self {
            InkSet::CMYK => 1,
            InkSet::NotCMYK => 2,
        };
        Some(TIFFValue::Short(vec![val]))
    }
}

short_value! {
    #[doc = "The number of inks. Usually equal to SamplesPerPixel, unless there are extra samples."]
    NumberOfInks,
    Tag::NumberOfInks
}

impl Default for NumberOfInks {
    fn default() -> NumberOfInks {
        NumberOfInks(4)
    }
}

ascii_value! {
    #[doc = "The name of each ink used in a separated"]
    InkNames,
    Tag::InkNames
}

ascii_value! {
    #[doc = "A description of the printing environment for which this separation is intended."]
    TargetPrinter,
    Tag::TargetPrinter
}
