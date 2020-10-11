use crate::Serializable;
use std::io::{Seek, Read, Write};
use crate::access::FieldAccessFlags;
use crate::constantpool::CPIndex;

#[derive(Clone, Debug, PartialEq)]
pub struct Field {
	access_flags: FieldAccessFlags,
	name_index: CPIndex,
	descriptor_index: CPIndex,
	attributes_count: CPIndex
}

impl Serializable for Field {
	fn parse<R: Seek + Read>(rdr: &mut R) -> Self {
		unimplemented!()
	}
	
	fn write<W: Seek + Write>(&self, wtr: &mut W) {
		unimplemented!()
	}
}
