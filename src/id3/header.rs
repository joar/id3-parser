use anyhow::anyhow;
use binrw::{BinRead, BinReaderExt};
use modular_bitfield::bitfield;
use modular_bitfield::prelude::*;

use crate::error::Error;
use crate::id3::helper::parse_id3_4x7_bit_be_uint;

#[bitfield]
#[derive(BinRead, Debug, Eq, PartialEq)]
#[br(map = Self::from_bytes)]
pub struct HeaderFlags {
    is_unsynchronized: bool,
    extended_header: bool,
    experimental_indicator: bool,
    footer_present: bool,
    rest: B4,
}

/// https://mutagen-specs.readthedocs.io/en/latest/id3/id3v2.4.0-structure.html#id3v2-header
#[derive(BinRead, Debug, Eq, PartialEq)]
#[br(magic = b"ID3", assert(version != 0xff), assert(revision != 0xff))]
pub struct Header {
    /// Major version
    /// - Will never be `\xff`
    #[br(big)]
    version: u8,
    /// Revision number
    /// - Will never be `\xff`
    /// - All revisions are backwards compatible while major versions are not.
    #[br(big)]
    revision: u8,
    /// The first bit (bit 7) in the ‘ID3 flags’ is indicating whether or not
    /// unsynchronisation is used (see section 5 for details); a set bit indicates usage.
    #[br(big)]
    flags: HeaderFlags,
    /// Size
    /// - Each byte Must be less than \x80
    ///
    /// The ID3 tag size is encoded with four bytes where the first bit (bit 7) is set to zero in
    /// every byte, making a total of 28 bits. The zeroed bits are ignored, so a 257 bytes long
    /// tag is represented as $00 00 02 01.
    ///
    /// The ID3 tag size is the size of the complete tag after unsychronisation, including padding,
    /// excluding the header (total tag size - 10). The reason to use 28 bits (representing up to
    /// 256MB) for size description is that we don’t want to run out of space here.
    #[br(big, parse_with = parse_id3_4x7_bit_be_uint)]
    size: u32,
}

/// https://mutagen-specs.readthedocs.io/en/latest/id3/id3v2.4.0-structure.html#extended-header
pub struct ExtendedHeader {}

impl Header {
    pub fn new(version: u8, revision: u8, flags: HeaderFlags, size: u32) -> Header {
        Header {
            version,
            revision,
            flags,
            size,
        }
    }

    pub fn read<T: BinReaderExt>(mut reader: T) -> anyhow::Result<Self> {
        match reader.read_ne::<Header>() {
            Ok(id3) => Ok(id3),
            Err(err) => match err.custom_err::<Error>() {
                Some(e) => Err(anyhow!(*e)),
                _ => Err(err.into()),
            },
        }
    }
}

mod tests {
    use binrw::{io::Cursor, BinReaderExt};

    use crate::id3::header::Header;
    use crate::id3::HeaderFlags;

    #[test]
    fn test_parse_size() {
        let mut reader = Cursor::new(b"ID3\x02\x03\x01\0\0\x01\x00");
        let id3: Header = reader.read_ne().unwrap();
        assert_eq!(
            id3,
            Header::new(0x02, 0x03, HeaderFlags::from_bytes([0x01]), 0b10_00_00_00,)
        )
    }

    #[test]
    fn test_happy_parse() {
        let reader = Cursor::new(b"ID3\x02\x03\x01\0\0\x00\x01");
        let id3: Header = Header::read(reader).unwrap();
        assert_eq!(
            id3,
            Header::new(0x02, 0x03, HeaderFlags::from_bytes([0x01]), 0x1,)
        )
    }

    #[test]
    fn test_revision_ff() {
        let reader = Cursor::new(b"ID3\x03\xff\x01\0\0\0\0");
        let header = Header::read(reader);
        assert!(header.is_err());
    }

    #[test]
    fn test_version_ff() {
        let reader = Cursor::new(b"ID3\xff\x00\x01\0\0\0\0");
        let header = Header::read(reader);

        assert!(header.is_err());
    }

    #[test]
    fn test_bad_size() {
        let reader = Cursor::new(b"ID3\x04\x00\x01\0\0\0\xff");
        let header = Header::read(reader);
        assert!(header.is_err());
    }
}
