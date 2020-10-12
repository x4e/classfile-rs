use crate::access::MethodAccessFlags;
use crate::attributes::{Attribute, Attributes, AttributeSource};
use crate::version::ClassVersion;
use crate::constantpool::ConstantPool;
use std::io::{Seek, Read, Write};
use crate::Serializable;
use byteorder::{BigEndian, ReadBytesExt};

#[allow(non_snake_case)]
pub mod Methods {
	use std::io::{Seek, Read, Write};
	use crate::method::Method;
	use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
	use crate::version::ClassVersion;
	use crate::constantpool::ConstantPool;
	
	pub fn parse<T: Seek + Read>(rdr: &mut T, version: &ClassVersion, constant_pool: &ConstantPool) -> Vec<Method> {
		let num_fields = rdr.read_u16::<BigEndian>().unwrap() as usize;
		let mut fields: Vec<Method> = Vec::with_capacity(num_fields);
		for _ in 0..num_fields {
			fields.push(Method::parse(rdr, version, constant_pool));
		}
		fields
	}
	
	pub fn write<T: Seek + Write>(wtr: &mut T, fields: &Vec<Method>, constant_pool: &ConstantPool) {
		wtr.write_u16::<BigEndian>(fields.len() as u16).unwrap();
		for field in fields.iter() {
			field.write(wtr, constant_pool);
		}
	}
}

#[derive(Clone, Debug, PartialEq)]
pub struct Method {
	access_flags: MethodAccessFlags,
	name: String,
	descriptor: String,
	attributes: Vec<Attribute>
}

impl Method {
	pub fn parse<R: Seek + Read>(rdr: &mut R, version: &ClassVersion, constant_pool: &ConstantPool) -> Self {
		let access_flags = MethodAccessFlags::parse(rdr);
		let name = constant_pool.utf8(rdr.read_u16::<BigEndian>().unwrap()).unwrap().str.clone();
		let descriptor = constant_pool.utf8(rdr.read_u16::<BigEndian>().unwrap()).unwrap().str.clone();
		let attributes = Attributes::parse(rdr, AttributeSource::Method, version, constant_pool);
		
		Method {
			access_flags,
			name,
			descriptor,
			attributes
		}
	}
	
	pub fn write<W: Seek + Write>(&self, wtr: &mut W, constant_pool: &ConstantPool) {
		self.access_flags.write(wtr);
		Attributes::write(wtr, &self.attributes, constant_pool);
	}
}
