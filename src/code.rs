use crate::attributes::{Attribute, AttributeSource, Attributes};
use crate::constantpool::ConstantPool;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Seek, Write};
use crate::version::ClassVersion;
use crate::error::Result;

#[derive(Clone, Debug, PartialEq)]
pub struct CodeAttribute {
	pub max_stack: u16,
	pub max_locals: u16,
	pub code: Vec<u8>,
	pub exceptions: Vec<ExceptionHandler>,
	pub attributes: Vec<Attribute>
}

impl CodeAttribute {
	pub fn parse(version: &ClassVersion, constant_pool: &ConstantPool, buf: Vec<u8>) -> Result<Self> {
		let mut slice = buf.as_slice();
		let max_stack = slice.read_u16::<BigEndian>()?;
		let max_locals = slice.read_u16::<BigEndian>()?;
		let code_length = slice.read_u32::<BigEndian>()?;
		let mut code: Vec<u8> = vec![0; code_length as usize];
		slice.read_exact(&mut code)?;
		let num_exceptions = slice.read_u16::<BigEndian>()?;
		let mut exceptions: Vec<ExceptionHandler> = Vec::with_capacity(num_exceptions as usize);
		for _ in 0..num_exceptions {
			exceptions.push(ExceptionHandler::parse(constant_pool, &mut slice)?);
		}
		let attributes = Attributes::parse(&mut slice, AttributeSource::Code, version, constant_pool)?;
		
		Ok(CodeAttribute {
			max_stack,
			max_locals,
			code,
			exceptions,
			attributes
		})
	}
	
	pub fn write<T: Seek + Write>(&self, wtr: &mut T, _constant_pool: &ConstantPool) -> Result<()> {
		wtr.write_u16::<BigEndian>(0)?; // write name
		wtr.write_u32::<BigEndian>(2)?; // length
		wtr.write_u16::<BigEndian>(0)?; // cp ref
		Ok(())
	}
}


#[derive(Clone, Debug, PartialEq)]
pub struct ExceptionHandler {
	pub start_pc: u16,
	pub end_pc: u16,
	pub handler_pc: u16,
	pub catch_type: Option<String>
}

impl ExceptionHandler {
	pub fn parse(constant_pool: &ConstantPool, buf: &mut &[u8]) -> Result<Self> {
		let start_pc = buf.read_u16::<BigEndian>()?;
		let end_pc = buf.read_u16::<BigEndian>()?;
		let handler_pc = buf.read_u16::<BigEndian>()?;
		let catch_index = buf.read_u16::<BigEndian>()?;
		let catch_type = if catch_index > 0 {
			Some(constant_pool.utf8(constant_pool.class(catch_index)?.name_index)?.str.clone())
		} else {
			None
		};
		
		Ok(ExceptionHandler {
			start_pc,
			end_pc,
			handler_pc,
			catch_type
		})
	}
	
	pub fn write<T: Seek + Write>(&self, wtr: &mut T, _constant_pool: &ConstantPool) -> Result<()> {
		wtr.write_u16::<BigEndian>(self.start_pc)?;
		wtr.write_u16::<BigEndian>(self.end_pc)?;
		wtr.write_u16::<BigEndian>(self.handler_pc)?;
		wtr.write_u16::<BigEndian>(0)?; // catch type cp ref
		Ok(())
	}
}
