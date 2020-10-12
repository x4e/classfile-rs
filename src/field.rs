use crate::Serializable;
use std::io::{Seek, Read, Write};
use crate::access::FieldAccessFlags;
use crate::constantpool::{CPIndex, ConstantPool};
use byteorder::{ReadBytesExt, BigEndian};
use crate::classfile::ClassFile;

#[derive(Clone, Debug, PartialEq)]
pub struct Field {
	access_flags: FieldAccessFlags,
	name: String,
	descriptor: String
}

impl Field {
	pub fn parse<R: Seek + Read>(rdr: &mut R, constant_pool: &ConstantPool) -> Self {
		let access_flags = FieldAccessFlags::parse(rdr);
		let name = constant_pool.utf8(rdr.read_u16::<BigEndian>().unwrap()).unwrap().str.clone();
		let descriptor = constant_pool.utf8(rdr.read_u16::<BigEndian>().unwrap()).unwrap().str.clone();
		let num_attributes = rdr.read_u16::<BigEndian>().unwrap();
		Field {
			access_flags,
			name,
			descriptor
		}
	}
	
	pub fn write<W: Seek + Write>(&self, wtr: &mut W) {
		self.access_flags.write(wtr);
	}
}
