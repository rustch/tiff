use std::io::Write;

use std::collections::HashMap;

use super::{TIFF_BE, TIFF_LE};

use endian::Endian;
use tag::{Field, Tag};
use value::TIFFValue;
error_chain ! {
    foreign_links {
        Io(::std::io::Error);
    }

    errors {
        EncodingError
    }
}
pub struct TIFFWriter<W> {
    inner: W,
    endian: Endian,
    values_map: HashMap<Tag, TIFFValue>,
    position: usize,
}

impl<W: Write> TIFFWriter<W> {
    pub fn new<T>(inner: W, endian: Endian) -> TIFFWriter<W> {
        TIFFWriter {
            inner,
            values_map: HashMap::new(),
            position: 0 as usize,
            endian,
        }
    }

    pub fn set_field<F: Field>(&mut self, f: &F) -> Result<()> {
        let value = match f.encode_to_value() {
            Some(val) => val,
            None => return Err(ErrorKind::EncodingError.into()),
        };

        self.values_map.insert(F::tag(), value);
        Ok(())
    }

    pub fn write(&mut self) -> Result<()> {
        self.write_header()
    }

    fn write_header(&mut self) -> Result<()> {
        // Order byte value
        let order_bytes = match self.endian {
            Endian::Little => TIFF_LE,
            Endian::Big => TIFF_BE,
        };

        self.inner.write_all(&order_bytes.to_bytes())?;
        self.position += 2;

        let magic_byte = match self.endian {
            Endian::Little => 42u8.to_le().to_bytes(),
            Endian::Big => 42u8.to_be().to_bytes(),
        };

        self.inner.write_all(&magic_byte)?;
        self.position += 2;
        Ok(())
    }

    fn adjuste_writer_to_next_ifd(&mut self) -> Result<()> {
        if self.position + 1 > (1 << 32 - 1) {
            Ok(())
        }
        if self.position % 2 != 0 {
            self.inner.write_all(&[0])?;
            self.position += 1;
        }
        Ok(())
    }
}
