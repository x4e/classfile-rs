use crate::Serializable;
use std::io::{Read, Write};
use crate::access::FieldAccessFlags;
use crate::constantpool::{ConstantPool, ConstantPoolWriter};
use byteorder::{ReadBytesExt, BigEndian};
use crate::attributes::{Attributes, Attribute, AttributeSource};
use crate::version::ClassVersion;
use crate::error::Result;
use crate::utils::mut_retain;

#[allow(non_snake_case)]
pub mod Fields {
	use std::io::{Read, Write};
	use crate::field::Field;
	use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
	use crate::version::ClassVersion;
	use crate::constantpool::{ConstantPool, ConstantPoolWriter};
	
	pub fn parse<T: Read>(rdr: &mut T, version: &ClassVersion, constant_pool: &ConstantPool) -> crate::Result<Vec<Field>> {
		let num_fields = rdr.read_u16::<BigEndian>()? as usize;
		let mut fields: Vec<Field> = Vec::with_capacity(num_fields);
		for _ in 0..num_fields {
			fields.push(Field::parse(rdr, version, constant_pool)?);
		}
		Ok(fields)
	}
	
	pub fn write<T: Write>(wtr: &mut T, fields: &Vec<Field>, constant_pool: &mut ConstantPoolWriter) -> crate::Result<()> {
		wtr.write_u16::<BigEndian>(fields.len() as u16)?;
		for field in fields.iter() {
			field.write(wtr, constant_pool)?;
		}
		Ok(())
	}
}

#[derive(Clone, Debug, PartialEq)]
pub struct Field {
	pub access_flags: FieldAccessFlags,
	pub name: String,
	pub descriptor: String,
	pub signature: Option<String>,
	pub attributes: Vec<Attribute>
}

impl Field {
	pub fn parse<R: Read>(rdr: &mut R, version: &ClassVersion, constant_pool: &ConstantPool) -> Result<Self> {
		let access_flags = FieldAccessFlags::parse(rdr)?;
		let name = constant_pool.utf8(rdr.read_u16::<BigEndian>()?)?.str.clone();
		let descriptor = constant_pool.utf8(rdr.read_u16::<BigEndian>()?)?.str.clone();
		let mut signature: Option<String> = None;
		let mut attributes = Attributes::parse(rdr, AttributeSource::Field, version, constant_pool)?;
		
		mut_retain(&mut attributes, |attribute| {
			if let Attribute::Signature(signature_attr) = attribute {
				// The attribute will be dropped, so instead of cloning we can swap an empty string for the signature
				#[allow(invalid_value)]
				let mut rep = String::new();
				std::mem::swap(&mut rep, &mut signature_attr.signature);
				signature = Some(rep);
				false
			} else {
				true
			}
		});
		
		Ok(Field {
			access_flags,
			name,
			descriptor,
			signature,
			attributes
		})
	}
	
	pub fn write<W: Write>(&self, wtr: &mut W, constant_pool: &mut ConstantPoolWriter) -> Result<()> {
		self.access_flags.write(wtr)?;
		Attributes::write(wtr, &self.attributes, constant_pool)?;
		Ok(())
	}
}
