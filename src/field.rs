use crate::Serializable;
use crate::access::FieldAccessFlags;
use crate::constantpool::{ConstantPool, ConstantPoolWriter};
use crate::attributes::{Attributes, Attribute, AttributeSource, SignatureAttribute};
use crate::version::ClassVersion;
use crate::error::Result;
use crate::utils::{VecUtils};
use std::io::{Read, Write};
use byteorder::{ReadBytesExt, BigEndian, WriteBytesExt};

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
	pub attributes: Vec<Attribute>
}

impl Field {
	pub fn parse<R: Read>(rdr: &mut R, version: &ClassVersion, constant_pool: &ConstantPool) -> Result<Self> {
		let access_flags = FieldAccessFlags::parse(rdr)?;
		let name = constant_pool.utf8(rdr.read_u16::<BigEndian>()?)?.str.clone();
		let descriptor = constant_pool.utf8(rdr.read_u16::<BigEndian>()?)?.str.clone();
		let attributes = Attributes::parse(rdr, AttributeSource::Field, version, constant_pool)?;
		
		Ok(Field {
			access_flags,
			name,
			descriptor,
			attributes
		})
	}
	
	pub fn signature(&mut self) -> Option<&mut String> {
		for attr in self.attributes.iter_mut() {
			if let Attribute::Signature(sig) = attr {
				return Some(&mut sig.signature)
			}
		}
		return None
	}
	
	pub fn set_signature(&mut self, sig: Option<String>) {
		// According to the JVM spec there must be at most one signature attribute in the attributes table
		// first find the index of the existing sig
		let index = self.attributes.find_first(|attr| {
			if let Attribute::Signature(_) = attr { true } else { false }
		});
		if let Some(sig) = sig {
			let attr = Attribute::Signature(SignatureAttribute::new(sig.clone()));
			if let Some(index) = index {
				self.attributes.replace(index, attr);
			} else {
				self.attributes.push(attr);
			}
		} else if let Some(index) = index {
			self.attributes.remove(index);
		}
	}
	
	pub fn write<W: Write>(&self, wtr: &mut W, constant_pool: &mut ConstantPoolWriter) -> Result<()> {
		self.access_flags.write(wtr)?;
		wtr.write_u16::<BigEndian>(constant_pool.utf8(self.name.clone()))?;
		wtr.write_u16::<BigEndian>(constant_pool.utf8(self.descriptor.clone()))?;
		Attributes::write(wtr, &self.attributes, constant_pool)?;
		Ok(())
	}
}
