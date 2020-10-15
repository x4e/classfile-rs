use thiserror::Error;
use std::{io, result};
use std::fmt::{Debug};

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
    None(),
    #[error("unimplemented")]
    Unimplemented
}
pub type Result<T> = result::Result<T, ParserError>;
