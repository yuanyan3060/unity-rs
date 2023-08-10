use std::io;
use thiserror::Error;

#[derive(Debug, Error)]

pub enum DecodeImageError {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error("cannot decode Image")]
    ImageDecode,
}
