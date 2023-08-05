use thiserror::Error;
#[derive(Error, Debug)]
pub enum UnityError{
    #[error("Eof")]
    Eof,
    #[error("Utf8")]
    Utf8(#[from] std::str::Utf8Error),
    #[error("InvalidValue")]
    InvalidValue,
    #[error("Lz4DecompressError: {0}")]
    Lz4DecompressError(#[from] lz4_flex::block::DecompressError),
    #[error("LzmaError: {0}")]
    LzmaError(#[from] lzma_rs::error::Error),
    #[error("Unimplemented")]
    Unimplemented,
}

pub type UnityResult<T> = Result<T, UnityError>;