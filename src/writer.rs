use std::io::Write;

use std::collections::HashMap;

use super::{TIFF_BE, TIFF_LE};

use endian::Endian;
use tag::{Field, Tag};
use value::TIFFValue;

error_chain! {
    foreign_links {
        Io(::std::io::Error);
    }

    errors {
        EncodingError
        OutOfBounds
    }
}

struct WritingEntryPayload {
    tag: Tag,
    count: usize,
    payload: Vec<u8>,
    value_type: u16,
}

pub struct TIFFWriter<W> {
    inner: W,
    endian: Endian,
    ifds: Vec<HashMap<Tag, WritingEntryPayload>>,
    position: usize,
}

fn write_ifd_tag<'a, W: Write>(
    w: &mut W,
    position: usize,
    endian: Endian,
    ifd: Vec<&'a WritingEntryPayload>,
) -> Result<Vec<&'a WritingEntryPayload>> {
    // Sort tag by value
    let mut buff: [u8; 12] = [0; 12];
    let mut big_entries = Vec::new();
    let mut next_data_cursor = position + ifd.len() * 12 + 4; // +4 For the next offset

    for entry in ifd {
        // Writing

        // 1 - Tag
        let tag = endian.short_adjusted(entry.tag.tag_value());
        buff.copy_from_slice(&tag);

        // 2 - Type
        let value_type = endian.short_adjusted(entry.value_type);
        buff[2..4].copy_from_slice(&value_type);

        // 3 - Count
        let count = endian.short_adjusted(entry.count as u16);
        buff[4..6].copy_from_slice(&count);

        // 4 - Offset/Value
        if entry.payload.len() <= 4 {
            buff[6..8].copy_from_slice(&entry.payload);
        } else {
            // We need to compute the offset with the provided parameters
            buff[6..8].copy_from_slice(&endian.short_adjusted(next_data_cursor as u16));
            next_data_cursor += entry.payload.len();
            big_entries.push(entry);
        }
    }
    w.write_all(&buff)?;
    Ok(big_entries)
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

        let entry = match value.convert_to_entry(F::tag(), self.endian) {
            Ok(val) => val,
            Err(_err) => return Err(ErrorKind::EncodingError.into()),
        };
        self.ifds[index].insert(F::tag(), entry);
        Ok(())
    }

    pub fn write(&mut self) -> Result<()> {
        // Header
        self.write_header_magic()?;

        // First 0th Offset -> 8
        self.inner.write_all(&self.endian.long_adjusted(8u32))?;
        self.position += 4;

        for ifd in &self.ifds {
            // Adjust position
            if self.position + 1 > (1 << (32 - 1)) {
                return Ok(());
            }

            if self.position % 2 != 0 {
                self.inner.write_all(&[0])?;
                self.position += 1;
            }

            // Write ifd len
            self.inner
                .write_all(&self.endian.short_adjusted(ifd.len() as u16))?;
            self.position += 2;

            // Sort tag by value
            let mut sorted_tags = ifd.keys().collect::<Vec<&Tag>>();
            sorted_tags.sort_by(|a, b| a.tag_value().cmp(&b.tag_value()));

            // Create list to apply elements
            let entries: Vec<&WritingEntryPayload> =
                ifd.into_iter().map(|(_, value)| value).collect();

            // Write IFD tag list
            let big_values = write_ifd_tag(&mut self.inner, self.position, self.endian, entries)?;

            let all_big: Vec<u8> = big_values
                .iter()
                .flat_map(|v| &v.payload)
                .cloned()
                .collect();
            // write_ifd_bigvalues(&mut self.inner, self.endian, &big_values_entries)?;
            self.inner.write_all(&all_big)?;
        }
        Ok(())
    }

    fn write_header_magic(&mut self) -> Result<()> {
        // Order byte value
        let order_bytes = match self.endian {
            Endian::Little => TIFF_LE,
            Endian::Big => TIFF_BE,
        };

        self.inner
            .write_all(&self.endian.short_adjusted(order_bytes))?;
        self.position += 2;

        let magic_byte = match self.endian {
            Endian::Little => 42u16.to_le_bytes(),
            Endian::Big => 42u16.to_be_bytes(),
        };

        self.inner.write_all(&magic_byte)?;
        self.position += 2;
        Ok(())
    }
}

impl TIFFValue {
    fn convert_to_entry(self, tag: Tag, endian: Endian) -> Result<WritingEntryPayload> {
        let value_type: u16 = match self {
            TIFFValue::Byte(_) => 1,
            TIFFValue::Ascii(_) => 2,
            TIFFValue::Short(_) => 3,
            TIFFValue::Long(_) => 4,
            TIFFValue::Rational(_) => 5,
            TIFFValue::SByte(_) => 6,
            TIFFValue::Undefined(_) => 7,
            TIFFValue::SShort(_) => 8,
            TIFFValue::SLong(_) => 9,
            TIFFValue::SRational(_) => 10,
            TIFFValue::Float(_) => 11,
            TIFFValue::Double(_) => 12,
        };

        let payload: (usize, Vec<u8>) = match self {
            TIFFValue::Byte(val) => (val.len(), val),
            TIFFValue::Ascii(val) => {
                if val.iter().all(|s| s[..].is_ascii()) {
                    return Err(ErrorKind::EncodingError.into());
                }

                let size = val.len();
                let content = val.into_iter().flat_map(|s| s.into_bytes()).collect();
                (size, content)
            }
            TIFFValue::Short(val) => {
                let len = val.len();
                let mut buff = Vec::new();
                for el in val {
                    buff.extend_from_slice(&endian.short_adjusted(el));
                }
                (len, buff)
            }
            TIFFValue::Long(val) => {
                let len = val.len();
                let mut buff = Vec::new();
                for el in val {
                    buff.extend_from_slice(&endian.long_adjusted(el));
                }
                (len, buff)
            }
            TIFFValue::Rational(val) => {
                let len = val.len();
                let mut buff = Vec::new();
                for el in val {
                    buff.extend_from_slice(&endian.long_adjusted(el.num));
                    buff.extend_from_slice(&endian.long_adjusted(el.denom));
                }
                (len, buff)
            }
            TIFFValue::SByte(val) => {
                let len = val.len();
                let mut buff = Vec::new();
                for el in val {
                    buff.extend_from_slice(&endian.byte_adjusted(el));
                }
                (len, buff)
            }
            TIFFValue::Undefined(val) => (val.len(), val),
            TIFFValue::SShort(val) => {
                let len = val.len();
                let mut buff = Vec::new();
                for el in val {
                    buff.extend_from_slice(&endian.short_adjusted(el));
                }
                (len, buff)
            }
            TIFFValue::SLong(val) => {
                let len = val.len();
                let mut buff = Vec::new();
                for el in val {
                    buff.extend_from_slice(&endian.long_adjusted(el));
                }
                (len, buff)
            }
            TIFFValue::SRational(val) => {
                let len = val.len();
                let mut buff = Vec::new();
                for el in val {
                    buff.extend_from_slice(&endian.long_adjusted(el.num));
                    buff.extend_from_slice(&endian.long_adjusted(el.denom));
                }
                (len, buff)
            }
            TIFFValue::Float(val) => {
                let len = val.len();
                let mut buff = Vec::new();
                for el in val {
                    buff.extend_from_slice(&endian.long_adjusted(el.to_bits()));
                }
                (len, buff)
            }
            TIFFValue::Double(val) => {
                let len = val.len();
                let mut buff = Vec::new();
                for el in val {
                    buff.extend_from_slice(&endian.longlong_adjusted(el.to_bits()));
                }
                (len, buff)
            }
        };

        Ok(WritingEntryPayload {
            count: payload.0,
            payload: payload.1,
            value_type,
            tag,
        })
    }
}
