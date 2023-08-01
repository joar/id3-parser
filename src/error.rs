use thiserror::Error;

#[derive(Error, Debug, Copy, Clone)]
pub enum Error {
    #[error("invalid ID3 size byte (pos: {idx}, value: {value})")]
    InvalidId3Size { idx: usize, value: u8 },
}
