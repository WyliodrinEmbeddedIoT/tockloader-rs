use byteorder::{ByteOrder, LittleEndian};
#[derive(Debug)]
#[allow(dead_code)]
pub struct TBFHeader {
    tbf_version: u16,
    header_size: u16,
    total_size: u32,
    flag: u32,
    checksum: u32,
}

impl TBFHeader {
    pub fn new(buf: Vec<u8>) -> Self {
        let tbf_version = LittleEndian::read_u16(&buf[0..2]);

        let header_size = LittleEndian::read_u16(&buf[2..4]);

        let total_size = LittleEndian::read_u32(&buf[4..8]);

        let flag = LittleEndian::read_u32(&buf[8..12]);

        let checksum = LittleEndian::read_u32(&buf[12..16]);

        TBFHeader {
            tbf_version,
            header_size,
            total_size,
            flag,
            checksum,
        }
    }
}
