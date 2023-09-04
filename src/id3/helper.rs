use crate::error::Error;
use binrw::{binrw, BinReaderExt, BinResult, BinWrite, BinWriterExt};

/// From the spec:
///
/// > The ID3 tag size is encoded with four bytes where the first bit (bit 7) is set to zero in
/// > every byte, making a total of 28 bits. The zeroed bits are ignored, so a 257 bytes long
/// > tag is represented as $00 00 02 01.
/// >
/// > The ID3 tag size is the size of the complete tag after unsychronisation, including padding,
/// > excluding the header (total tag size - 10). The reason to use 28 bits (representing up to
/// > 256MB) for size description is that we don’t want to run out of space here.
/// > -- https://mutagen-specs.readthedocs.io/en/latest/id3/id3v2.2.html#id3v2-header
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

/// Inspired by https://phoxis.org/2010/05/08/synch-safe/
#[binrw::writer(writer, endian)]
pub fn write_id3_4x7_bit_be_uint(size: &u32) -> BinResult<()> {
    let mut size_remaining: u32 = *size;
    const NUM_BYTES: usize = 4;
    let mut bytes: [u8; NUM_BYTES] = [0, 0, 0, 0];

    for (i, mut byte) in bytes.iter_mut().enumerate() {
        dbg!(format!("{size_remaining:b}"));
        let shift = 7 * (i);
        let size_after_shift = (size_remaining >> (shift));
        let u7ish = (size_after_shift & 0x7f) as u8;
        size_remaining = size - u7ish as u32;
        *byte = u7ish;
        dbg!(shift, format!("{size_after_shift:b}"), u7ish);
    }
    dbg!(bytes);
    bytes.write_options(writer, endian, ())?;
    Ok(())
}

pub fn parse_utf8_string(bytes: Vec<u8>) -> anyhow::Result<String> {
    Ok(String::from_utf8(bytes)?)
}

pub fn string_to_utf8_bytes(str: &String) -> anyhow::Result<Vec<u8>> {
    Ok(str.as_bytes().to_vec())
}

mod tests {
    use crate::id3::helper::write_id3_4x7_bit_be_uint;
    use binrw::{BinWrite, BinWriterExt};
    use std::io::Cursor;

    #[derive(BinWrite)]
    #[brw(big)]
    struct SyncSafeInt {
        #[bw(write_with = write_id3_4x7_bit_be_uint)]
        value: u32,
    }

    #[test]
    fn test_write_id3_4x7_bit_be_uint() {
        let x: u32 = 0b1010_10101010_10101010;
        let container = SyncSafeInt { value: x };
        let mut writer = Cursor::new(Vec::new());
        container.write_be(&mut writer).unwrap();
        assert_eq!(writer.into_inner(), vec![0xff])
    }

    #[test]
    fn test_sync_safe_int_symmetric() {
        let x: u32 = 0b1010_10101010_10101010;
        let container = SyncSafeInt { value: x };
        let mut writer = Cursor::new(Vec::new());
        container.write_be(&mut writer).unwrap();
        let mut reader = Cursor::new(writer.into_inner());
        let decoded_container = q
    }
}
