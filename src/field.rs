use crate::Serializable;
use std::io::{Seek, Read, Write};
use crate::access::FieldAccessFlags;
use crate::constantpool::{ConstantPool};
use byteorder::{ReadBytesExt, BigEndian};
use crate::attributes::{Attributes, Attribute, AttributeSource};
use crate::version::ClassVersion;

#[allow(non_snake_case)]
pub mod Fields {
	use std::io::{Seek, Read, Write};
	use crate::field::Field;
	use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
	use crate::version::ClassVersion;
	use crate::constantpool::ConstantPool;
	
	pub fn parse<T: Seek + Read>(rdr: &mut T, version: &ClassVersion, constant_pool: &ConstantPool) -> Vec<Field> {
		let num_fields = rdr.read_u16::<BigEndian>().unwrap() as usize;
		let mut fields: Vec<Field> = Vec::with_capacity(num_fields);
		for _ in 0..num_fields {
			fields.push(Field::parse(rdr, version, constant_pool));
		}
		fields
	}
	
	pub fn write<T: Seek + Write>(wtr: &mut T, fields: &Vec<Field>, constant_pool: &ConstantPool) {
		wtr.write_u16::<BigEndian>(fields.len() as u16).unwrap();
		for field in fields.iter() {
			field.write(wtr, constant_pool);
		}
	}
}

#[derive(Clone, Debug, PartialEq)]
pub struct Field {
	access_flags: FieldAccessFlags,
	name: String,
	descriptor: String,
	attributes: Vec<Attribute>
}

impl Field {
	pub fn parse<R: Seek + Read>(rdr: &mut R, version: &ClassVersion, constant_pool: &ConstantPool) -> Self {
		let access_flags = FieldAccessFlags::parse(rdr);
		let name = constant_pool.utf8(rdr.read_u16::<BigEndian>().unwrap()).unwrap().str.clone();
		let descriptor = constant_pool.utf8(rdr.read_u16::<BigEndian>().unwrap()).unwrap().str.clone();
		let attributes = Attributes::parse(rdr, AttributeSource::Field, version, constant_pool);
		
		Field {
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
