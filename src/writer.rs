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

pub struct TIFFWriter {
    write_buff: Vec<u8>,
    endian: Endian,
    ifds: Vec<HashMap<Tag, WritingEntryPayload>>,
    position: usize,
    current_index: usize,
}

fn write_ifd_tag<'a>(
    out_buff: &mut Vec<u8>,
    position: usize,
    endian: Endian,
    ifd: Vec<&'a WritingEntryPayload>,
) -> Vec<&'a WritingEntryPayload> {
    // Sort tag by value
    let mut big_entries = Vec::new();
    let mut next_data_cursor = position + ifd.len() * 12 + 4; // +4 For the next offset
    let mut tag_data = Vec::new();

    for entry in ifd {
        // Writing
        // 1 - Tag
        let tag = endian.short_adjusted(entry.tag.tag_value());
        tag_data.extend_from_slice(&tag);

        // 2 - Type
        let value_type = endian.short_adjusted(entry.value_type);
        tag_data.extend_from_slice(&value_type);

        // 3 - Count
        let count = endian.long_adjusted(entry.count as u32);
        tag_data.extend_from_slice(&count);

        // 4 - Offset/Value
        let diff = 4i16 - (entry.payload.len() as i16);
        if diff >= 0 {
            tag_data.extend_from_slice(&entry.payload);
            if diff > 0 {
                // 0 are on left for le(aka left justified)
                let vec = vec![0; diff as usize];
                tag_data.extend_from_slice(&vec);
            }
        } else {
            // We need to compute the offset with the provided parameters
            tag_data.extend_from_slice(&endian.long_adjusted(next_data_cursor as u32));
            next_data_cursor += entry.payload.len();
            big_entries.push(entry);
        }
        out_buff.append(&mut tag_data);
    }

    big_entries
}

impl TIFFWriter {
    /// Creates a new writer with a provided `Endian` with one directory to write in.
    pub fn new(endian: Endian) -> TIFFWriter {
        TIFFWriter {
            write_buff: Vec::new(),
            ifds: vec![HashMap::new()],
            position: 0 as usize,
            current_index: 0 as usize,
            endian,
        }
    }

    /// Creates a new directory at the provided index
    pub fn insert_directory_at_index(&mut self, ifd: usize) {
        if ifd > self.ifds.len() {
            panic!("Out of range")
        }
        self.ifds.insert(ifd, HashMap::new());
    }

    pub fn set_current_directory_index(&mut self, index: usize) {
        if index > self.ifds.len() - 1 {
            panic!("Out of range")
        }
        self.current_index = index;
    }

    pub fn set_field<F: Field>(&mut self, f: &F) -> Result<()> {
        let value = match f.encode_to_value() {
            Some(val) => val,
            None => return Err(ErrorKind::EncodingError.into()),
        };

        self.set_directory_field_for_tag(F::tag(), value)
    }

    pub fn set_directory_field_for_tag(&mut self, tag: Tag, value: TIFFValue) -> Result<()> {
        let entry = match value.convert_to_entry(tag, self.endian) {
            Ok(val) => val,
            Err(_err) => return Err(ErrorKind::EncodingError.into()),
        };
        self.ifds[self.current_index].insert(tag, entry);
        Ok(())
    }

    pub fn write<W: Write>(&mut self, f: &mut W) -> Result<()> {
        // Header
        self.write_header_magic()?;

        // First 0th Offset -> 8
        self.write_buff
            .extend_from_slice(&self.endian.long_adjusted(8u32));
        self.position += 4;

        for (index, ifd) in self.ifds.iter().enumerate() {
            // Adjust position
            if self.position + 1 > (1 << (32 - 1)) {
                return Ok(());
            }

            if self.position % 2 != 0 {
                self.write_buff.extend_from_slice(&[0]);
                self.position += 1;
            }

            // Write ifd len
            self.write_buff
                .extend_from_slice(&self.endian.short_adjusted(ifd.len() as u16));
            self.position += 2;

            // Sort tag by value
            let mut sorted_entries: Vec<_> = ifd.iter().collect();
            sorted_entries.sort_by(|a, b| a.0.tag_value().cmp(&b.0.tag_value()));

            // Write IFD
            let entries: Vec<&WritingEntryPayload> =
                sorted_entries.into_iter().map(|(_, value)| value).collect();
            let entries_size = entries.len() * 12;
            let big_values =
                write_ifd_tag(&mut self.write_buff, self.position, self.endian, entries);
            self.position += entries_size;

            // Write data
            let mut all_big: Vec<u8> = big_values
                .iter()
                .flat_map(|v| &v.payload)
                .cloned()
                .collect();

            // Write Next Offset
            let mut next_available_space = if index == self.ifds.len() - 1 {
                0
            } else {
                self.position + all_big.len() + 1
            };

            if next_available_space % 2 != 0 {
                next_available_space += 1;
            }

            let next_offset = &self.endian.long_adjusted(next_available_space as u32);
            self.write_buff.extend_from_slice(next_offset);
            self.position += next_offset.len();

            // write_ifd_bigvalues(&mut self.inner, self.endian, &big_values_entries)?;
            self.write_buff.append(&mut all_big);

            self.position += all_big.len();
        }
        f.write_all(&self.write_buff)
            .map_err(|e| ErrorKind::Io(e).into())
    }

    fn write_header_magic(&mut self) -> Result<()> {
        // Order byte value
        let order_bytes = match self.endian {
            Endian::Little => TIFF_LE,
            Endian::Big => TIFF_BE,
        };

        self.write_buff
            .extend_from_slice(&self.endian.short_adjusted(order_bytes));
        self.position += 2;

        let magic_byte = match self.endian {
            Endian::Little => 42u16.to_le_bytes(),
            Endian::Big => 42u16.to_be_bytes(),
        };

        self.write_buff.extend_from_slice(&magic_byte);
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

#[cfg(test)]
mod tests {
    use super::TIFFWriter;
    use reader::TIFFReader;
    use std::fs::File;
    use std::io::Cursor;

    #[test]

    fn test_read_write() {
        let bytes: &[u8] = include_bytes!("../samples/arbitro_be.tiff");
        let mut in_cursor = Cursor::new(bytes);
        let mut read = TIFFReader::new(&mut in_cursor).unwrap();

        let mut writer = TIFFWriter::new(read.endianness());

        for i in 0..read.directories_count() {
            if i > 1 {
                read.set_directory_index(i);
                writer.insert_directory_at_index(i);
                writer.set_current_directory_index(i);
            }

            let tags = read.get_directory_tags();
            for tag in tags {
                let value = read.get_directory_value_from_tag(tag).unwrap();
                println!("{:?}", value);
                writer.set_directory_field_for_tag(tag, value).unwrap();
            }
        }
        let mut file = File::create("test_output.tiff").unwrap();
        writer.write(&mut file).unwrap();
    }
}
