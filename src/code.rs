use crate::attributes::{Attribute, AttributeSource, Attributes};
use crate::constantpool::ConstantPool;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Seek, Write};
use crate::version::ClassVersion;

#[derive(Clone, Debug, PartialEq)]
pub struct CodeAttribute {
	pub max_stack: u16,
	pub max_locals: u16,
	pub code: Vec<u8>,
	pub exceptions: Vec<ExceptionHandler>,
	pub attributes: Vec<Attribute>
}

impl CodeAttribute {
	pub fn parse(version: &ClassVersion, constant_pool: &ConstantPool, buf: Vec<u8>) -> Self {
		let mut slice = buf.as_slice();
		let max_stack = slice.read_u16::<BigEndian>().unwrap();
		let max_locals = slice.read_u16::<BigEndian>().unwrap();
		let code_length = slice.read_u32::<BigEndian>().unwrap();
		let mut code: Vec<u8> = Vec::with_capacity(code_length as usize);
		slice.take(code_length as u64).read_to_end(&mut code);
		let num_exceptions = slice.read_u16::<BigEndian>().unwrap();
		let mut exceptions: Vec<ExceptionHandler> = Vec::with_capacity(num_exceptions as usize);
		for _ in 0..num_exceptions {
			exceptions.push(ExceptionHandler::parse(constant_pool, &mut slice));
		}
		let attributes = Attributes::parse(&mut slice, AttributeSource::Code, version, constant_pool);
		
		CodeAttribute {
			max_stack,
			max_locals,
			code,
			exceptions,
			attributes
		}
	}
	
	pub fn write<T: Seek + Write>(&self, wtr: &mut T, _constant_pool: &ConstantPool) {
		wtr.write_u16::<BigEndian>(0).unwrap(); // write name
		wtr.write_u32::<BigEndian>(2).unwrap(); // length
		wtr.write_u16::<BigEndian>(0).unwrap(); // cp ref
	}
}


#[derive(Clone, Debug, PartialEq)]
pub struct ExceptionHandler {
	pub start_pc: u16,
	pub end_pc: u16,
	pub handler_pc: u16,
	pub catch_type: String
}

impl ExceptionHandler {
	pub fn parse(constant_pool: &ConstantPool, buf: &mut &[u8]) -> Self {
		let start_pc = buf.read_u16::<BigEndian>().unwrap();
		let end_pc = buf.read_u16::<BigEndian>().unwrap();
		let handler_pc = buf.read_u16::<BigEndian>().unwrap();
		let catch_type = constant_pool.utf8(constant_pool.class(buf.read_u16::<BigEndian>().unwrap()).unwrap().name_index).unwrap().str.clone();
		
		ExceptionHandler {
			start_pc,
			end_pc,
			handler_pc,
			catch_type
		}
	}
	
	pub fn write<T: Seek + Write>(&self, wtr: &mut T, _constant_pool: &ConstantPool) {
		wtr.write_u16::<BigEndian>(self.start_pc).unwrap();
		wtr.write_u16::<BigEndian>(self.end_pc).unwrap();
		wtr.write_u16::<BigEndian>(self.handler_pc).unwrap();
		wtr.write_u16::<BigEndian>(0).unwrap(); // catch type cp ref
	}
}
