use std::io::{Read, Seek, SeekFrom};

use reader;
use reader::TIFFReader;
use tag::*;
use value::Rational;

error_chain! {
    links {
        Reader(reader::Error, reader::ErrorKind);
    }
    errors {
        MissingData {}
        InvalidBaseline {}
    }
}


pub struct Image<R> {
    inner: TIFFReader<R>,
    width: ImageWidth,
    length: ImageLength,
    compression: Compression,
    photometric_interpretation: PhotometricInterpretation,
    x_resolution: XResolution,
    y_resolution: YResolution,
    stripes_offsets: StripOffsets,
    stripes_bytes_count: StripByteCounts,
    resolution_unit: ResolutionUnit,
}

impl<R: Read + Seek> Image<R> {
    pub fn new(reader: R) -> Result<Image<R>> {
        let mut inner = TIFFReader::new(reader)?;

        let width = inner
            .get_directory_field::<ImageWidth>()
            .ok_or(ErrorKind::MissingData)?;

        let length = inner
            .get_directory_field::<ImageLength>()
            .ok_or(ErrorKind::MissingData)?;

        let compression = inner
            .get_directory_field::<Compression>()
            .ok_or(ErrorKind::MissingData)?;

        let photometric_interpretation = inner
            .get_directory_field::<PhotometricInterpretation>()
            .ok_or(ErrorKind::MissingData)?;

        
        let x_resolution = inner
            .get_directory_field::<XResolution>()
            .ok_or(ErrorKind::MissingData)?;

        let y_resolution = inner
            .get_directory_field::<YResolution>()
            .ok_or(ErrorKind::MissingData)?;

        let stripes_offsets = inner
            .get_directory_field::<StripOffsets>()
            .ok_or(ErrorKind::MissingData)?;

        let stripes_bytes_count = inner
            .get_directory_field::<StripByteCounts>()
            .ok_or(ErrorKind::MissingData)?;

        let resolution_unit = inner
            .get_directory_field::<ResolutionUnit>()
            .ok_or(ErrorKind::MissingData)?;

        Ok(Image {
            inner,
            width,
            length,
            compression,
            photometric_interpretation,
            x_resolution,
            y_resolution,
            stripes_offsets,
            stripes_bytes_count,
            resolution_unit
        })
    }

    pub fn stripes_iter(self) -> StripesIter<R> {
        StripesIter {
            image: self,
            index: 0,
        }
    }
}

pub struct Baseline<R> {
    baseline: Image<R>,
}

impl<R: Read + Seek> Baseline<R> {
    pub fn new(reader: R) -> Result<Baseline<R>> {
        let baseline = Image::new(reader)?;

        if baseline.photometric_interpretation != PhotometricInterpretation::WhiteIsZero || baseline.photometric_interpretation != PhotometricInterpretation::BlackIsZero {
            return Err(ErrorKind::InvalidBaseline.into());
        }

        Ok(Baseline { baseline })
    }
}

pub struct Grayscale<R> {
    baseline: Image<R>,
    bit_per_sample: BitsPerSample,
}

impl<R: Read + Seek> Grayscale<R> {
     pub fn new(reader: R) -> Result<Grayscale<R>> {
         let image = Image::new(reader)?;

         
     }
}

pub struct StripesIter<R> {
    image: Image<R>,
    index: usize,
}

impl<R: Read + Seek> Iterator for StripesIter<R> {
    type Item = Vec<u8>;
    fn next(&mut self) -> Option<Vec<u8>> {
        if self.index >= self.image.stripes_bytes_count.0.len() {
            return None;
        }

        let reader = self.image.inner.reader_as_ref();
        let offset = u64::from(self.image.stripes_offsets.0[self.index]);
        let count = self.image.stripes_bytes_count.0[self.index] as usize;

        let mut buff = vec![0; count];

        reader.seek(SeekFrom::Start(offset)).ok()?;
        reader.read_exact(&mut buff).ok()?;

        self.index += 1;
        Some(buff)
    }
}

#[cfg(test)]
mod tests {
    use super::Image;
    use std::io::Cursor;

    #[test]
    fn test_read_baseline() {
        let bytes: &[u8] = include_bytes!("../../samples/ycbcr-cat.tif");
        let mut cursor = Cursor::new(bytes);
        let image = Image::new(&mut cursor).expect("Should be a valid baseline image");
        let stripes: Vec<Vec<u8>> = image.stripes_iter().collect();
        assert!(!stripes.is_empty());
    }
}
