use thiserror::Error;
use std::{io, result};
use std::fmt::{Debug};
use crate::constantpool::ConstantType;
use std::str::Utf8Error;
use std::string::FromUtf8Error;

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("Error reading/writing")]
    IO(io::Error),
    #[error("Incompatible Constant Entry (expected {expected:#?} at {index:?})")]
    IncompatibleCPEntry {
        expected: &'static str,
        found: ConstantType,
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
    #[error("Invalid Instruction {pc} {msg}")]
    InvalidInstruction {
        pc: u32,
	    msg: String
    },
    #[error("Unimplemented {0}")]
    Unimplemented(&'static str),
	#[error("Out of bounds jump index {0}")]
	OutOfBoundsJumpIndex(i32),
	#[error("Invalid Utf8 {0}")]
	InvalidUtf8(Utf8Error),
	#[error("Too many instructions in method")]
	TooManyInstructions(),
	#[error("Invalid Descriptor: {0}")]
	InvalidDescriptor(String),
	#[error("{0}")]
	Other(String)
}

impl ParserError {
	fn check_panic(self) -> Self {
		if let Ok(x) = std::env::var("PANIC_ON_ERR") {
			if x == "1" || x == "true" {
				panic!("{:#x?}", self)
			}
		}
		self
	}
	
	pub fn io(inner: io::Error) -> Self {
		ParserError::IO(inner).check_panic()
	}
	
	pub fn incomp_cp(expected: &'static str, found: &ConstantType, index: usize) -> Self {
		ParserError::IncompatibleCPEntry {
			expected,
			found: found.clone(),
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
	
	pub fn invalid_insn<T>(pc: u32, msg: T) -> Self
		where T: Into<String> {
		ParserError::InvalidInstruction {
			pc,
			msg: msg.into()
		}
	}
	
	pub fn unimplemented(name: &'static str) -> Self {
		ParserError::Unimplemented(name).check_panic()
	}
	
	pub fn out_of_bounds_jump(index: i32) -> Self {
		ParserError::OutOfBoundsJumpIndex(index).check_panic()
	}
	
	pub fn invalid_utf8(err: Utf8Error) -> Self {
		ParserError::InvalidUtf8(err).check_panic()
	}
	
	pub fn too_many_instructions() -> Self {
		ParserError::TooManyInstructions().check_panic()
	}
	
	pub fn invalid_descriptor<T: Into<String>>(msg: T) -> Self {
		ParserError::InvalidDescriptor(msg.into()).check_panic()
	}
	
	#[inline]
	pub fn other<T>(name: T) -> Self
		where T: Into<String> {
		ParserError::Other(name.into()).check_panic()
	}
	
	
	
	
	pub fn unmapped_label() -> Self {
		ParserError::other("No mapping found for label")
	}
}

impl From<io::Error> for ParserError {
	fn from(inner: io::Error) -> Self {
		ParserError::io(inner)
	}
}

impl From<std::string::FromUtf8Error> for ParserError {
	fn from(err: FromUtf8Error) -> Self {
		ParserError::invalid_utf8(err.utf8_error())
	}
}

pub type Result<T> = result::Result<T, ParserError>;
