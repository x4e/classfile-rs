use thiserror::Error;
use std::{io, result};
use std::fmt::{Debug};

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("Error reading/writing")]
    IO(#[from] io::Error),
    #[error("Incompatible Constant Entry (expected {expected:?} at {index:?})")]
    IncompatibleCPEntry {
        expected: &'static str,
        found: String,
        index: usize
    },
    #[error("Unrecognized {0}: {1}")]
    Unrecognized(&'static str, String),
    #[error("Invalid constant pool index: {0}")]
    Index(usize),
    #[error("Something was none!")]
    None(),
    #[error("Unknown Instruction {opcode:X}")]
    UnknownInstruction {
	    opcode: u8
    },
    #[error("Unimplemented {name:?}")]
    Unimplemented {
	    name: &'static str
    },
	#[error("{name:?}")]
	Other {
		name: &'static str
	}
}
pub type Result<T> = result::Result<T, ParserError>;
