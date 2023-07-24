use std::fmt;
use std::fs::File;
use std::io::{Read, Seek};

use binread::{BinRead, BinReaderExt, BinResult, ReadOptions};
use binread::io::StreamPosition;
use modular_bitfield::bitfield;
use modular_bitfield::prelude::B7;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("invalid ID3 size byte (pos: {idx}, value: {value})")]
    InvalidId3Size {idx: u32, value: u8}
}

fn parse_id3_size<R: Read + Seek>(reader: &mut R, ro: &ReadOptions, _: ())
                                  -> BinResult<u32>
{
    let mut sum: u32 = 0;
    for i in  0..4 {
        let b: u8 = reader.read_be().unwrap();
        if b >= 0b1000_0000 {
            return Err(binread::Error::Custom {pos: reader.stream_pos().unwrap(), err: Box::new(format!("{}", Error::InvalidId3Size { idx: i, value: b}))});
        }
        sum = sum + b as u32;
    }
    Ok(sum)
}

#[bitfield]
#[derive(BinRead, Debug, Eq, PartialEq)]
#[br(map = Self::from_bytes)]
struct ID3Flags {
    is_unsynchronized: bool,
    rest: B7,
}

/// https://mutagen-specs.readthedocs.io/en/latest/id3/id3v2.2.html#id3v2-header
#[derive(BinRead, Debug, Eq, PartialEq)]
#[br(magic = b"ID3", assert(version != 0xff), assert(revision != 0xff))]
struct ID3 {
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
    flags: ID3Flags,
    /// Size
    ///
    /// The ID3 tag size is encoded with four bytes where the first bit (bit 7) is set to zero in
    /// every byte, making a total of 28 bits. The zeroed bits are ignored, so a 257 bytes long
    /// tag is represented as $00 00 02 01.
    ///
    /// The ID3 tag size is the size of the complete tag after unsychronisation, including padding,
    /// excluding the header (total tag size - 10). The reason to use 28 bits (representing up to
    /// 256MB) for size description is that we don’t want to run out of space here.
    #[br(big, parse_with = parse_id3_size)]
    size: u32,
}

const SEVEN_BIT_MAX: u8 = 0xF0;

fn main() {
    let mut reader = File::open("./fq/format/mp3/testdata/header-zeros-frames.mp3").unwrap();
    // let mut reader: Cursor<Vec<&u8>> = Cursor::new(b"ID3\x02\x03\x01".to_vec().iter().chain(&[SEVEN_BIT_MAX; 4].to_vec()).collect());
    let id3: ID3 = reader.read_ne().unwrap();
    println!("id3: {:?}", id3);
}

mod tests {
    use std::process::id;
    use binread::{BinReaderExt, Error, io::Cursor};

    use crate::{ID3, ID3Flags};

    #[test]
    fn test_happy_parse() {
        let mut reader = Cursor::new(b"ID3\x02\x03\x01\0\0\x02\x01");
        let id3: ID3 = reader.read_ne().unwrap();
        assert_eq!(id3, ID3 {
            version: 0x02,
            revision: 0x03,
            flags: ID3Flags { bytes: [0x01] },
            size: 0x3,
        })
    }

    #[test]
    fn test_revision_ff() {
        let mut reader = Cursor::new(b"ID3\x03\xff\x01\0\0\0\0");
        let id3: Result<ID3, Error> = reader.read_ne();
        assert!(id3.is_err());
    }

    #[test]
    fn test_version_ff() {
        let mut reader = Cursor::new(b"ID3\xff\x00\x01\0\0\0\0");
        let id3: Result<ID3, Error> = reader.read_ne();
        assert!(id3.is_err());
    }

    #[test]
    fn test_bad_size() {
        let mut reader = Cursor::new(b"ID3\x02\x00\x01\0\0\0\xff");
        let id3: Result<ID3, Error> = reader.read_ne();
        assert!(id3.is_err());
    }
}
