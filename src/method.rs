use crate::access::MethodAccessFlags;
use crate::attributes::{Attribute, Attributes, AttributeSource, SignatureAttribute, ExceptionsAttribute};
use crate::version::ClassVersion;
use crate::constantpool::{ConstantPool, ConstantPoolWriter};
use crate::Serializable;
use crate::error::Result;
use crate::utils::{VecUtils};
use crate::code::CodeAttribute;
use std::io::{Read, Write};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

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
	pub attributes: Vec<Attribute>
}

impl Method {
	pub fn parse<R: Read>(rdr: &mut R, version: &ClassVersion, constant_pool: &ConstantPool) -> Result<Self> {
		let access_flags = MethodAccessFlags::parse(rdr)?;
		let name = constant_pool.utf8(rdr.read_u16::<BigEndian>()?)?.str.clone();
		let descriptor = constant_pool.utf8(rdr.read_u16::<BigEndian>()?)?.str.clone();
		
		let attributes = Attributes::parse(rdr, AttributeSource::Method, version, constant_pool)?;
		
		let meth = Method {
			access_flags,
			name,
			descriptor,
			attributes
		};
		Ok(meth)
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
	
	pub fn exceptions(&mut self) -> Option<&mut Vec<String>> {
		for attr in self.attributes.iter_mut() {
			if let Attribute::Exceptions(x) = attr {
				return Some(&mut x.exceptions)
			}
		}
		return None
	}
	
	pub fn set_exceptions(&mut self, exc: Option<Vec<String>>) {
		let index = self.attributes.find_first(|attr| {
			if let Attribute::Exceptions(_) = attr { true } else { false }
		});
		if let Some(exc) = exc {
			let attr = Attribute::Exceptions(ExceptionsAttribute::new(exc.clone()));
			if let Some(index) = index {
				self.attributes.replace(index, attr);
			} else {
				self.attributes.push(attr);
			}
		} else if let Some(index) = index {
			self.attributes.remove(index);
		}
	}
	
	pub fn code(&mut self) -> Option<&mut CodeAttribute> {
		for attr in self.attributes.iter_mut() {
			if let Attribute::Code(x) = attr {
				return Some(x)
			}
		}
		return None
	}
	
	pub fn set_code(&mut self, code: Option<CodeAttribute>) {
		let index = self.attributes.find_first(|attr| {
			if let Attribute::Code(_) = attr { true } else { false }
		});
		if let Some(code) = code {
			let attr = Attribute::Code(code);
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
