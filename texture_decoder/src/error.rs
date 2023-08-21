use std::io;
use thiserror::Error;

#[derive(Debug, Error)]

pub enum DecodeImageError {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error("cannot decode Image")]
    ImageDecode,
    #[error("expect {0} times pixel decode, but need {1} times")]
    SizeNotMatch(usize, usize),
}
