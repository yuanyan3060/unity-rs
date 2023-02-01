#![allow(unused)]
#[derive(Debug)]
pub enum Error {
    EOF,
    Unimplemented,
    InvalidValue,
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::EOF => write!(f, "read eof"),
            Self::Unimplemented => write!(f, "this file is not support"),
            Self::InvalidValue => write!(f, "invalid value"),
        }
    }
}

impl std::error::Error for Error {
    
}
