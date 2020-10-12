use crate::Serializable;
use std::io::{Seek, Read, Write};
use crate::access::FieldAccessFlags;
use crate::constantpool::{ConstantPool};
use byteorder::{ReadBytesExt, BigEndian};
use crate::attributes::{Attributes};
use crate::version::ClassVersion;

#[derive(Clone, Debug, PartialEq)]
pub struct Field {
	access_flags: FieldAccessFlags,
	name: String,
	descriptor: String,
	attributes: Attributes
}

#[allow(dead_code)]
impl Field {
	pub fn parse<R: Seek + Read>(rdr: &mut R, version: &ClassVersion, constant_pool: &ConstantPool) -> Self {
		let access_flags = FieldAccessFlags::parse(rdr);
		let name = constant_pool.utf8(rdr.read_u16::<BigEndian>().unwrap()).unwrap().str.clone();
		let descriptor = constant_pool.utf8(rdr.read_u16::<BigEndian>().unwrap()).unwrap().str.clone();
		let attributes = Attributes::parse_field(rdr, version, constant_pool);
		
		Field {
			access_flags,
			name,
			descriptor,
			attributes
		}
	}
	
	pub fn write<W: Seek + Write>(&self, wtr: &mut W) {
		self.access_flags.write(wtr);
		self.attributes.write(wtr);
	}
}
