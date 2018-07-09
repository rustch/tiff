use ifd::IFDType;

#[derive(Debug)]
pub enum Tag {
    PhotometricInterpretation,
    Compression,
    ImageLength,
    ImageWidth,
    ResolutionUnit,
    XResolution,
    YResolution,
    RowsPerStrip,
    StripsOffsets,
    StripByteCounts,
    BitsPerSample,
    ColorMap,
    SamplesPerPixel,
    Artist,
    CellLength,
    CellWidth,
    Copyright,
    DateTime,
    ExtraSamples,
    FillOrder
}