use std::io::{Seek, Read, Write};
use crate::constantpool::ConstantPool;
use byteorder::{ReadBytesExt, BigEndian};
use crate::version::ClassVersion;

#[derive(Clone, Debug, PartialEq)]
pub struct Attributes {
	attributes: Vec<Attribute>
}

impl Attributes {
	pub fn parse_field<R: Seek + Read>(rdr: &mut R, version: &ClassVersion, constant_pool: &ConstantPool) -> Self {
		let num_attributes = rdr.read_u16::<BigEndian>().unwrap() as usize;
		let mut attributes: Vec<Attribute> = Vec::with_capacity(num_attributes);
		for i in 0..num_attributes {
			let name = constant_pool.utf8(rdr.read_u16::<BigEndian>().unwrap()).unwrap().str.clone();
			let attribute_length = rdr.read_u32::<BigEndian>().unwrap() as usize;
			let mut buf: Vec<u8> = Vec::with_capacity(attribute_length);
			rdr.take(attribute_length as u64).read_to_end(&mut buf);
			attributes.push(Attribute::Unknown(UnknownAttribute { name, buf }));
		}
		
		Attributes {
			attributes
		}
	}
	
	pub fn write<W: Seek + Write>(&self, _wtr: &mut W) {
		unimplemented!()
	}
}

#[derive(Clone, Debug, PartialEq)]
pub struct UnknownAttribute {
	pub name: String,
	pub buf: Vec<u8>
}

#[derive(Clone, Debug, PartialEq)]
pub enum Attribute {
	Unknown(UnknownAttribute)
}
