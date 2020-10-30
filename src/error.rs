use thiserror::Error;
use std::{io, result};
use std::fmt::{Debug};

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("Error reading/writing")]
    IO(io::Error),
    #[error("Incompatible Constant Entry (expected {expected:?} at {index:?})")]
    IncompatibleCPEntry {
        expected: &'static str,
        found: String,
        index: usize
    },
    #[error("Unrecognized {0}: {1}")]
    Unrecognized(&'static str, String),
    #[error("Invalid constant pool index: {0}")]
    BadCpIndex(u16),
    #[error("{0} was none!")]
    None(&'static str),
    #[error("Unknown Instruction {opcode:X}")]
    UnknownInstruction {
	    opcode: u8
    },
    #[error("Unimplemented {0}")]
    Unimplemented(&'static str),
	#[error("Out of bounds jump index {0}")]
	OutOfBoundsJumpIndex(i32),
	#[error("{0}")]
	Other(&'static str)
}

impl ParserError {
	fn check_panic(self) -> Self {
		if let Ok(x) = std::env::var("PANIC_ON_ERR") {
			if x == "1" || x == "true" {
				panic!("{:#?}", self)
			}
		}
		self
	}
	
	pub fn io(inner: io::Error) -> Self {
		ParserError::IO(inner).check_panic()
	}
	
	pub fn incomp_cp(expected: &'static str, found: String, index: usize) -> Self {
		ParserError::IncompatibleCPEntry {
			expected,
			found,
			index
		}.check_panic()
	}
	
	pub fn unrecognised(first: &'static str, second: String) -> Self {
		ParserError::Unrecognized(first, second).check_panic()
	}
	
	pub fn bad_cp_index<T>(index: T) -> Self
		where T: Into<u16> {
		ParserError::BadCpIndex(index.into()).check_panic()
	}
	
	pub fn none(name: &'static str) -> Self {
		ParserError::None(name).check_panic()
	}
	
	pub fn unknown_insn(opcode: u8) -> Self {
		ParserError::UnknownInstruction { opcode }.check_panic()
	}
	
	pub fn unimplemented(name: &'static str) -> Self {
		ParserError::Unimplemented(name).check_panic()
	}
	
	pub fn out_of_bounds_jump(index: i32) -> Self {
		ParserError::OutOfBoundsJumpIndex(index).check_panic()
	}
	
	pub fn other(name: &'static str) -> Self {
		ParserError::Other(name).check_panic()
	}
}

impl From<io::Error> for ParserError {
	fn from(inner: io::Error) -> Self {
		ParserError::io(inner)
	}
}

pub type Result<T> = result::Result<T, ParserError>;
