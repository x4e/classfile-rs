use crate::constantpool::{ConstantPool, ConstantType, ConstantPoolWriter};
use byteorder::{ReadBytesExt, BigEndian, WriteBytesExt};
use std::io::{Write, Read};
use crate::version::{MajorVersion, ClassVersion};
use crate::code::CodeAttribute;
use crate::error::Result;

#[allow(non_snake_case)]
pub mod Attributes {
	use std::io::{Read, Write};
	use crate::constantpool::{ConstantPool, ConstantPoolWriter};
	use byteorder::{ReadBytesExt, BigEndian, WriteBytesExt};
	use crate::version::{ClassVersion};
	use crate::attributes::{Attribute, AttributeSource};
	
	pub fn parse<R: Read>(rdr: &mut R, source: AttributeSource, version: &ClassVersion, constant_pool: &ConstantPool) -> crate::Result<Vec<Attribute>> {
		let num_attributes = rdr.read_u16::<BigEndian>()? as usize;
		let mut attributes: Vec<Attribute> = Vec::with_capacity(num_attributes);
		for _ in 0..num_attributes {
			attributes.push(Attribute::parse(rdr, &source, version, constant_pool)?);
		}
		Ok(attributes)
	}
	
	pub fn write<W: Write>(wtr: &mut W, attributes: &Vec<Attribute>, constant_pool: &mut ConstantPoolWriter) -> crate::Result<()> {
		wtr.write_u16::<BigEndian>(attributes.len() as u16)?;
		for attribute in attributes.iter() {
			attribute.write(wtr, constant_pool)?;
		}
		Ok(())
	}
}

#[derive(Clone, Debug, PartialEq)]
pub struct ConstantValueAttribute {
	value: ConstantValue
}

#[derive(Clone, Debug, PartialEq)]
pub enum ConstantValue {
	Long(i64),
	Float(f32),
	Double(f64),
	Int(i32),
	String(String)
}

impl ConstantValueAttribute {
	pub fn parse(constant_pool: &ConstantPool, buf: Vec<u8>) -> Result<Self> {
		let index = buf.as_slice().read_u16::<BigEndian>()?;
		let value = match constant_pool.get(index)? {
			ConstantType::Long(x) => ConstantValue::Long(x.bytes),
			ConstantType::Float(x) => ConstantValue::Float(x.bytes()),
			ConstantType::Double(x) => ConstantValue::Double(x.bytes()),
			ConstantType::Integer(x) => ConstantValue::Int(x.bytes),
			ConstantType::String(x) => ConstantValue::String(constant_pool.utf8(x.utf_index)?.str.clone()),
			x => panic!("Invalid constant value type {:#?} at index {}", x, index)
		};
		Ok(ConstantValueAttribute {
			value
		})
	}
	
	pub fn write<T: Write>(&self, wtr: &mut T, _constant_pool: &mut ConstantPoolWriter) -> Result<()> {
		wtr.write_u16::<BigEndian>(0)?; // write name
		wtr.write_u32::<BigEndian>(2)?; // length
		wtr.write_u16::<BigEndian>(0)?; // cp ref
		Ok(())
	}
}

#[derive(Clone, Debug, PartialEq)]
pub struct SignatureAttribute {
	pub signature: String
}

impl SignatureAttribute {
	pub fn parse(constant_pool: &ConstantPool, buf: Vec<u8>) -> Result<Self> {
		let index = buf.as_slice().read_u16::<BigEndian>()?;
		let signature = constant_pool.utf8(index)?.str.clone();
		Ok(SignatureAttribute {
			signature
		})
	}
	
	pub fn write<T: Write>(&self, wtr: &mut T, _constant_pool: &mut ConstantPoolWriter) -> Result<()> {
		wtr.write_u16::<BigEndian>(0)?; // write name
		wtr.write_u32::<BigEndian>(2)?; // length
		wtr.write_u16::<BigEndian>(0)?; // cp ref
		Ok(())
	}
}

#[derive(Clone, Debug, PartialEq)]
pub struct ExceptionsAttribute {
	pub exceptions: Vec<String>
}

impl ExceptionsAttribute {
	pub fn parse(constant_pool: &ConstantPool, buf: Vec<u8>) -> Result<Self> {
		let mut slice = buf.as_slice();
		let num_exceptions = slice.read_u16::<BigEndian>()?;
		let mut exceptions: Vec<String> = Vec::with_capacity(num_exceptions as usize);
		for _ in 0..num_exceptions {
			exceptions.push(constant_pool.utf8(constant_pool.class(slice.read_u16::<BigEndian>()?)?.name_index)?.str.clone());
		}
		Ok(ExceptionsAttribute {
			exceptions
		})
	}
	
	pub fn write<T: Write>(&self, wtr: &mut T, _constant_pool: &mut ConstantPoolWriter) -> Result<()> {
		wtr.write_u16::<BigEndian>(self.exceptions.len() as u16)?;
		for _exception in self.exceptions.iter() {
			// write exception
			wtr.write_u16::<BigEndian>(0)?;
		}
		Ok(())
	}
}

#[derive(Clone, Debug, PartialEq)]
pub struct UnknownAttribute {
	pub name: String,
	pub buf: Vec<u8>
}

#[derive(Clone, Debug, PartialEq)]
pub struct SourceFileAttribute {
	pub source_file: String
}

impl SourceFileAttribute {
	pub fn parse(constant_pool: &ConstantPool, buf: Vec<u8>) -> Result<Self> {
		let index = buf.as_slice().read_u16::<BigEndian>()?;
		let source_file = constant_pool.utf8(index)?.str.clone();
		Ok(SourceFileAttribute {
			source_file
		})
	}
	
	pub fn write<T: Write>(&self, wtr: &mut T, _constant_pool: &mut ConstantPoolWriter) -> Result<()> {
		wtr.write_u16::<BigEndian>(0)?; // write name
		wtr.write_u16::<BigEndian>(0)?; // write source file
		Ok(())
	}
}

impl UnknownAttribute {
	pub fn parse(name: String, buf: Vec<u8>) -> Result<Self> {
		Ok(UnknownAttribute {
			name, buf
		})
	}
	
	pub fn write<T: Write>(&self, wtr: &mut T, _constant_pool: &mut ConstantPoolWriter) -> Result<()> {
		wtr.write_u16::<BigEndian>(0)?; // write name
		wtr.write_u32::<BigEndian>(self.buf.len() as u32)?; // length
		wtr.write_all(self.buf.as_slice())?;
		Ok(())
	}
}

#[derive(Clone, Debug, PartialEq)]
pub enum Attribute {
	ConstantValue(ConstantValueAttribute),
	Signature(SignatureAttribute),
	Code(CodeAttribute),
	Exceptions(ExceptionsAttribute),
	SourceFile(SourceFileAttribute),
	Unknown(UnknownAttribute)
}

impl Attribute {
	pub fn parse<R: Read>(rdr: &mut R, source: &AttributeSource, version: &ClassVersion, constant_pool: &ConstantPool) -> Result<Attribute> {
		let name = constant_pool.utf8(rdr.read_u16::<BigEndian>()?)?.str.clone();
		let attribute_length = rdr.read_u32::<BigEndian>()? as usize;
		let mut buf: Vec<u8> = Vec::with_capacity(attribute_length);
		rdr.take(attribute_length as u64).read_to_end(&mut buf)?;
		let str = name.as_str();
		
		Ok(match source {
			AttributeSource::Class => {
				if str == "SourceFile" {
					Attribute::SourceFile(SourceFileAttribute::parse(constant_pool, buf)?)
				} else {
					Attribute::Unknown(UnknownAttribute::parse(name, buf)?)
				}
			},
			AttributeSource::Field => {
				if str == "ConstantValue" {
					Attribute::ConstantValue(ConstantValueAttribute::parse(constant_pool, buf)?)
				} else if str == "Signature" && version.major >= MajorVersion::JAVA_5 {
					Attribute::Signature(SignatureAttribute::parse(constant_pool, buf)?)
				} else {
					Attribute::Unknown(UnknownAttribute::parse(name, buf)?)
				}
			},
			AttributeSource::Method => {
				if str == "Code" {
					Attribute::Code(CodeAttribute::parse(version, constant_pool, buf)?)
				} else if str == "Signature" && version.major >= MajorVersion::JAVA_5 {
					Attribute::Signature(SignatureAttribute::parse(constant_pool, buf)?)
				} else if str == "Exceptions" {
					Attribute::Exceptions(ExceptionsAttribute::parse(constant_pool, buf)?)
				} else {
					Attribute::Unknown(UnknownAttribute::parse(name, buf)?)
				}
			}
			AttributeSource::Code => {
				Attribute::Unknown(UnknownAttribute::parse(name, buf)?)
			}
		})
	}
	
	pub fn write<T: Write>(&self, wtr: &mut T, constant_pool: &mut ConstantPoolWriter) -> Result<()> {
		match self {
			Attribute::ConstantValue(t) => t.write(wtr, constant_pool),
			Attribute::Signature(t) => t.write(wtr, constant_pool),
			Attribute::Code(t) => t.write(wtr, constant_pool),
			Attribute::Exceptions(t) => t.write(wtr, constant_pool),
			Attribute::SourceFile(t) => t.write(wtr, constant_pool),
			Attribute::Unknown(t) => t.write(wtr, constant_pool)
		}
	}
}

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum AttributeSource {
	Class,
	Field,
	Method,
	Code
}
