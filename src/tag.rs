use ifd::IFDValueType;
use std::convert::{From, Into};
use std::fmt::{Display, Error, Formatter};

macro_rules! tags_definition {
    {$(
        $name:ident | $value:expr => $desc:expr,
    )*} => {
        #[derive(Debug, Copy, Clone, Hash)]
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
                $( Tag::$name => { write!(f, "Tag $name ($value): $desc") })*,
                Tag::Unknown(value) => { write!(f, "Unkown value: {}", value) }
              }
          }
      }
    }
}

tags_definition! {
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
}
