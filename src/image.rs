use std::io::{Read, Seek, SeekFrom};

pub mod baseline {
    use super::*;
    use reader;
    use reader::TIFFReader;
    use tag::*;

    error_chain! {
        links {
            Reader(reader::Error, reader::ErrorKind);
        }
        errors {
            StripesInformationMissing
            InvalidStripesConfiguration
        }
    }
    pub struct Image<R> {
        inner: TIFFReader<R>,
        stripes_offsets: StripOffsets,
        stripes_bytes_count: StripByteCounts,
    }

    impl<R: Read + Seek> Image<R> {
        pub fn new(reader: R) -> Result<Image<R>> {
            let mut inner = TIFFReader::new(reader)?;

            let stripes_offsets = inner
                .get_directory_field::<StripOffsets>()
                .ok_or(ErrorKind::StripesInformationMissing)?;

            let stripes_bytes_count = inner
                .get_directory_field::<StripByteCounts>()
                .ok_or(ErrorKind::StripesInformationMissing)?;

            Ok(Image {
                inner,
                stripes_offsets,
                stripes_bytes_count,
            })
        }

        pub fn stripes_iter(self) -> StripesIter<R> {
            StripesIter {
                image: self,
                index: 0,
            }
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
            let offset = self.image.stripes_offsets.0[self.index] as u64;
            let count = self.image.stripes_bytes_count.0[self.index] as usize;

            let mut buff = vec![0; count];

            reader.seek(SeekFrom::Start(offset)).ok()?;
            reader.read_exact(&mut buff).ok()?;

            self.index += 1;
            Some(buff)
        }
    }
}
