use thiserror::Error;
use std::{io, result};
use std::option::NoneError;
use std::fmt::{Debug, Formatter};

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("error reading/writing")]
    IO(#[from] io::Error),
    #[error("Incompatible Constant Entry (expected {expected:?} at {index:?})")]
    IncompatibleCPEntry {
        expected: &'static str,
        index: usize
    },
    #[error("unrecognized {0}: {1}")]
    Unrecognized(&'static str, String),
    #[error("invalid index: {0}")]
    Index(usize),
    #[error("Something was none!")]
    None(NoneError),
    #[error("unimplemented")]
    Unimplemented
}
impl From<NoneError> for ParserError {
    fn from(e: NoneError) -> Self {
        Self::None(e)
    }
}
pub type Result<T> = result::Result<T, ParserError>;