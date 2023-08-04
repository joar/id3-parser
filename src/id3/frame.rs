use binrw::BinRead;
use modular_bitfield::bitfield;
use modular_bitfield::prelude::*;

use crate::id3::helper::{parse_id3_4x7_bit_be_uint, parse_utf8_string};

#[bitfield]
#[derive(BinRead, Debug, Eq, PartialEq)]
pub struct FrameFlags {
    foo: bool,
    rest: B7,
}

#[derive(BinRead, Debug, Eq, PartialEq)]
pub struct Frame {
    #[br(count = 4, try_map = parse_utf8_string)]
    identifier: String,
    #[br(big, parse_with = parse_id3_4x7_bit_be_uint)]
    size: u32,
}

mod tests {
    use binrw::io::Cursor;
    use binrw::BinReaderExt;

    use crate::id3::frame::Frame;

    #[test]
    fn test_read_tag() {
        let mut reader = Cursor::new(b"TIT2\0\0\0\x06");
        let tag = reader.read_be::<Frame>().unwrap();
        assert_eq!(
            tag,
            Frame {
                identifier: "TIT2".to_string(),
                size: 0x06
            }
        )
    }

    #[test]
    fn test_read_tag_2() {
        let mut reader = Cursor::new(b"TIT2\0\0\x01\x7f");
        let frame = reader.read_be::<Frame>().unwrap();
        assert_eq!(
            frame,
            Frame {
                identifier: "TIT2".to_string(),
                size: 0xff
            }
        )
    }
}
