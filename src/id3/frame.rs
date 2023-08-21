use binrw::{BinRead, BinWrite};
use modular_bitfield::bitfield;
use modular_bitfield::prelude::*;

use crate::id3::helper::{
    parse_id3_4x7_bit_be_uint, parse_utf8_string, string_to_utf8_bytes, write_id3_4x7_bit_be_uint,
};

#[bitfield]
#[derive(BinRead, Debug, Eq, PartialEq)]
#[br(map = Self::from_bytes)]
pub struct FrameStatus {
    unused: bool,
    tag_alter_preservation: bool,
    file_alter_preservation: bool,
    read_only: bool,
    rest: B4,
}

impl Default for FrameStatus {
    fn default() -> Self {
        FrameStatus::from_bytes([0x0])
    }
}

#[bitfield]
#[derive(BinRead, Debug, Eq, PartialEq)]
#[br(map = Self::from_bytes)]
pub struct FrameFormat {
    unused: bool,
    rest: B7,
}

impl Default for FrameFormat {
    fn default() -> Self {
        FrameFormat::from_bytes([0x0])
    }
}

#[derive(BinRead, BinWrite, Debug, Eq, PartialEq, Default)]
#[brw(big)]
pub struct Frame {
    #[br(count = 4, try_map = parse_utf8_string)]
    #[bw(try_map = string_to_utf8_bytes)]
    identifier: String,

    #[br(parse_with = parse_id3_4x7_bit_be_uint)]
    #[bw(write_with = write_id3_4x7_bit_be_uint)]
    size: u32,
    #[bw(map = |x| x.bytes)]
    status: FrameStatus,
    #[bw(map = |x| x.bytes)]
    format: FrameFormat,
}

mod tests {
    use binrw::io::Cursor;
    use binrw::{BinReaderExt, BinWrite, BinWriterExt};

    use crate::id3::frame::{Frame, FrameFormat};
    use crate::id3::FrameStatus;

    #[test]
    fn test_write_frame() {
        let frame = Frame {
            identifier: "TIT2".to_string(),
            size: 0,
            status: FrameStatus::new()
                .with_unused(false)
                .with_tag_alter_preservation(true)
                .with_file_alter_preservation(true)
                .with_read_only(false)
                .with_rest(0b0000),
            format: FrameFormat::new().with_unused(false).with_rest(0b0000000),
        };
        let mut writer = Cursor::new(Vec::new());
        frame.write(&mut writer).unwrap();
        let expected: Vec<u8> = b"TIT2"
            .to_vec()
            .iter()
            .map(ToOwned::to_owned)
            .chain(vec![0b0110_0000u8])
            .chain(vec![0b0000_0000u8])
            .collect();
        assert_eq!(writer.into_inner(), expected)
    }

    #[test]
    fn test_read_tag() {
        let mut reader = Cursor::new(b"TIT2\0\0\0\x06\0\0");
        let frame = reader.read_be::<Frame>().unwrap();
        assert_eq!(
            frame,
            Frame {
                identifier: "TIT2".to_string(),
                size: 0x06,
                ..Default::default()
            }
        )
    }

    #[test]
    fn test_read_tag_2() {
        let mut reader = Cursor::new(b"TIT2\0\0\x01\x7f\0\0");
        let frame = reader.read_be::<Frame>().unwrap();
        assert_eq!(
            frame,
            Frame {
                identifier: "TIT2".to_string(),
                size: 0xff,
                ..Default::default()
            }
        )
    }
}
