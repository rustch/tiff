use std::io::Write;

use std::collections::HashMap;

use super::{TIFF_BE, TIFF_LE};

use endian::Endian;
use ifd::TIFFValue;
use tag::{Field, Tag};

error_chain! {
    foreign_links {
        Io(::std::io::Error);
    }

    errors {
        EncodingError
        OutOfBounds
    }
}
pub struct TIFFWriter<W> {
    inner: W,
    endian: Endian,
    ifds: Vec<HashMap<Tag, TIFFValue>>,
    position: usize,
}

impl<W: Write> TIFFWriter<W> {
    pub fn new<T>(inner: W, endian: Endian) -> TIFFWriter<W> {
        TIFFWriter {
            inner,
            ifds: vec![HashMap::new()],
            position: 0 as usize,
            endian,
        }
    }

    pub fn create_new_directory_after(&mut self, ifd: usize) -> Result<usize> {
        if ifd >= self.ifds.len() {
            Err(ErrorKind::OutOfBounds.into())
        } else {
            self.ifds.insert(ifd, HashMap::new());
            Ok(self.ifds.len() - 1)
        }
    }

    pub fn set_field<F: Field>(&mut self, index: usize, f: &F) -> Result<()> {
        let value = match f.encode_to_value() {
            Some(val) => val,
            None => return Err(ErrorKind::EncodingError.into()),
        };

        self.ifds[index].insert(F::tag(), value);

        Ok(())
    }

    pub fn write(&mut self) -> Result<()> {
        self.write_header_magic()?;

        self.adjust_writer_to_next_ifd()?;

        for tags_map in &self.ifds {}

        Ok(())
    }

    fn write_ifd(&mut self, ifd: &HashMap<Tag, TIFFValue>) -> Result<()> {
        let mut sorted_tags = ifd.keys().collect::<Vec<&Tag>>();
        sorted_tags.sort_by(|a, b| a.tag_value().cmp(&b.tag_value()));

        for tag in sorted_tags {}

        Ok(())
    }

    fn write_header_magic(&mut self) -> Result<()> {
        // Order byte value
        let order_bytes = match self.endian {
            Endian::Little => TIFF_LE,
            Endian::Big => TIFF_BE,
        };

        self.inner.write_all(&order_bytes.to_bytes())?;
        self.position += 2;

        let magic_byte = match self.endian {
            Endian::Little => 42u16.to_le_bytes(),
            Endian::Big => 42u16.to_be_bytes(),
        };

        self.inner.write_all(&magic_byte)?;
        self.position += 2;
        Ok(())
    }

    fn adjust_writer_to_next_ifd(&mut self) -> Result<()> {
        if self.position + 1 > (1 << 32 - 1) {
            return Ok(());
        }

        if self.position % 2 != 0 {
            self.inner.write_all(&[0])?;
            self.position += 1;
        }
        Ok(())
    }
}
