use thiserror::Error;
#[derive(Error, Debug)]
pub enum UnityError {
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
    #[error("CustomError: {0}")]
    CustomError(String),
    #[error("DecodeImageError: {0}")]
    DecodeImage(#[from] texture_decoder::error::DecodeImageError),
    #[error("File type[{0:?}] support not implment yet")]
    UnsupportFileType(String),
    #[error("Image is zero sized")]
    ZeroSizeImage,
    #[error("Unimplemented")]
    Unimplemented,
}

pub type UnityResult<T> = Result<T, UnityError>;

impl From<&'static str> for UnityError {
    fn from(value: &'static str) -> Self {
        Self::CustomError(value.to_string())
    }
}
