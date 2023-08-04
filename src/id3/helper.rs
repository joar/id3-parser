use crate::error::Error;
use binrw::{BinReaderExt, BinResult};

/// From the spec:
///
/// > The ID3 tag size is encoded with four bytes where the first bit (bit 7) is set to zero in
/// > every byte, making a total of 28 bits. The zeroed bits are ignored, so a 257 bytes long
/// > tag is represented as $00 00 02 01.
/// >
/// > The ID3 tag size is the size of the complete tag after unsychronisation, including padding,
/// > excluding the header (total tag size - 10). The reason to use 28 bits (representing up to
/// > 256MB) for size description is that we don’t want to run out of space here.
#[binrw::parser(reader: reader)]
pub fn parse_id3_4x7_bit_be_uint() -> BinResult<u32> {
    let mut sum: u32 = 0;
    for i in 0..4 {
        let b: u8 = reader.read_be().unwrap();
        // Only the least significant 7 bits of each byte can be used
        if b >= 0x80u8 {
            // pls help I don't know how to do nice errors. I'm just stuffing a box of error in an
            // Any field here.
            return Err(binrw::Error::Custom {
                pos: reader.stream_position().unwrap(),
                err: Box::new(Error::InvalidId3Size { idx: i, value: b }),
            });
        }
        sum = sum << 7 | b as u32;
    }
    Ok(sum)
}

pub fn parse_utf8_string(bytes: Vec<u8>) -> anyhow::Result<String> {
    Ok(String::from_utf8(bytes)?)
}
