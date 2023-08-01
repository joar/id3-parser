use binrw::BinRead;

use crate::id3::helper::{parse_id3_7_byte_size, parse_utf8_string};

#[derive(BinRead, Debug, PartialEq)]
pub struct Frame {
    #[br(count = 4, try_map = parse_utf8_string)]
    identifier: String,
    #[br(big, parse_with = parse_id3_7_byte_size)]
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
}
