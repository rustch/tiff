#[derive(Debug)]
pub enum Endian {
    Big, Little
}

pub fn read_u16_from_endian(endian: &Endian, buf: [u8; 2]) -> u16 {
    let value = u16::from_bytes(buf);
    match endian {
        Endian::Big => { 
            u16::from_be(value)
        }
        Endian::Little => {
            u16::from_le(value)
        }
    }
}

pub fn read_u32_from_endian(endian: &Endian, buf: [u8; 4]) -> u32 {
        let value = u32::from_bytes(buf);
    match endian {
        Endian::Big => { 
            u32::from_be(value)
        }
        Endian::Little => {
            u32::from_le(value)
        }
    }
}