use crate::access::MethodAccessFlags;
use crate::attributes::{Attribute, Attributes, AttributeSource};
use crate::version::ClassVersion;
use crate::constantpool::{ConstantPool, ConstantPoolWriter};
use std::io::{Read, Write};
use crate::Serializable;
use byteorder::{BigEndian, ReadBytesExt};
use crate::error::Result;
use crate::utils::{mut_retain};
use crate::code::CodeAttribute;

#[allow(non_snake_case)]
pub mod Methods {
	use std::io::{Read, Write};
	use crate::method::Method;
	use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
	use crate::version::ClassVersion;
	use crate::constantpool::{ConstantPool, ConstantPoolWriter};
	
	pub fn parse<T: Read>(rdr: &mut T, version: &ClassVersion, constant_pool: &ConstantPool) -> crate::Result<Vec<Method>> {
		let num_fields = rdr.read_u16::<BigEndian>()? as usize;
		let mut fields: Vec<Method> = Vec::with_capacity(num_fields);
		for _ in 0..num_fields {
			fields.push(Method::parse(rdr, version, constant_pool)?);
		}
		Ok(fields)
	}
	
	pub fn write<T: Write>(wtr: &mut T, fields: &Vec<Method>, constant_pool: &mut ConstantPoolWriter) -> crate::Result<()> {
		wtr.write_u16::<BigEndian>(fields.len() as u16)?;
		for field in fields.iter() {
			field.write(wtr, constant_pool)?;
		}
		Ok(())
	}
}

#[derive(Clone, Debug, PartialEq)]
pub struct Method {
	pub access_flags: MethodAccessFlags,
	pub name: String,
	pub descriptor: String,
	pub signature: Option<String>,
	pub exceptions: Vec<String>,
	pub code: Option<CodeAttribute>,
	pub attributes: Vec<Attribute>
}

impl Method {
	pub fn parse<R: Read>(rdr: &mut R, version: &ClassVersion, constant_pool: &ConstantPool) -> Result<Self> {
		let access_flags = MethodAccessFlags::parse(rdr)?;
		let name = constant_pool.utf8(rdr.read_u16::<BigEndian>()?)?.str.clone();
		let descriptor = constant_pool.utf8(rdr.read_u16::<BigEndian>()?)?.str.clone();
		let mut signature: Option<String> = None;
		#[allow(invalid_value)]
		let mut exceptions: Vec<String> = Vec::new();
		let mut code: Option<CodeAttribute> = None;
		let mut attributes = Attributes::parse(rdr, AttributeSource::Method, version, constant_pool)?;
		
		mut_retain(&mut attributes, |attribute| {
			match attribute {
				Attribute::Signature(signature_attr) => {
					// The attribute will be dropped, so instead of cloning we can swap an empty string for the signature
					let mut rep = String::with_capacity(0);
					std::mem::swap(&mut rep, &mut signature_attr.signature);
					signature = Some(rep);
					false
				},
				Attribute::Exceptions(exceptions_attr) => {
					std::mem::swap(&mut exceptions, &mut exceptions_attr.exceptions);
					false
				},
				Attribute::Code(code_attr) => {
					let mut rep: CodeAttribute = CodeAttribute::empty();
					std::mem::swap(&mut rep, code_attr);
					code = Some(rep);
					false
				}
				_ => true
			}
		});
		
		Ok(Method {
			access_flags,
			name,
			descriptor,
			signature,
			exceptions,
			code,
			attributes
		})
	}
	
	pub fn write<W: Write>(&self, wtr: &mut W, constant_pool: &mut ConstantPoolWriter) -> Result<()> {
		self.access_flags.write(wtr)?;
		Attributes::write(wtr, &self.attributes, constant_pool)?;
		Ok(())
	}
}
