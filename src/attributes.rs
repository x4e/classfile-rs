use crate::constantpool::{ConstantPool, ConstantType};
use byteorder::{ReadBytesExt, BigEndian, WriteBytesExt};
use std::io::{Seek, Write, Read};
use crate::version::{MajorVersion, ClassVersion};
use crate::code::CodeAttribute;

#[allow(non_snake_case)]
pub mod Attributes {
	use std::io::{Seek, Read, Write};
	use crate::constantpool::{ConstantPool};
	use byteorder::{ReadBytesExt, BigEndian, WriteBytesExt};
	use crate::version::{ClassVersion};
	use crate::attributes::{Attribute, AttributeSource};
	
	pub fn parse<R: Read>(rdr: &mut R, source: AttributeSource, version: &ClassVersion, constant_pool: &ConstantPool) -> Vec<Attribute> {
		let num_attributes = rdr.read_u16::<BigEndian>().unwrap() as usize;
		let mut attributes: Vec<Attribute> = Vec::with_capacity(num_attributes);
		for _ in 0..num_attributes {
			attributes.push(Attribute::parse(rdr, &source, version, constant_pool));
		}
		attributes
	}
	
	pub fn write<W: Seek + Write>(wtr: &mut W, attributes: &Vec<Attribute>, constant_pool: &ConstantPool) {
		wtr.write_u16::<BigEndian>(attributes.len() as u16).unwrap();
		for attribute in attributes.iter() {
			attribute.write(wtr, constant_pool);
		}
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
	pub fn parse(constant_pool: &ConstantPool, buf: Vec<u8>) -> Self {
		let index = buf.as_slice().read_u16::<BigEndian>().unwrap();
		let value = match constant_pool.get(index) {
			Some(ConstantType::Long(x)) => ConstantValue::Long(x.bytes),
			Some(ConstantType::Float(x)) => ConstantValue::Float(x.bytes),
			Some(ConstantType::Double(x)) => ConstantValue::Double(x.bytes),
			Some(ConstantType::Integer(x)) => ConstantValue::Int(x.bytes),
			Some(ConstantType::String(x)) => ConstantValue::String(constant_pool.utf8(x.string_index).unwrap().str.clone()),
			x => panic!("Invalid constant value type {:#?} at index {}", x, index)
		};
		ConstantValueAttribute {
			value
		}
	}
	
	pub fn write<T: Seek + Write>(&self, wtr: &mut T, _constant_pool: &ConstantPool) {
		wtr.write_u16::<BigEndian>(0).unwrap(); // write name
		wtr.write_u32::<BigEndian>(2).unwrap(); // length
		wtr.write_u16::<BigEndian>(0).unwrap(); // cp ref
	}
}

#[derive(Clone, Debug, PartialEq)]
pub struct SignatureAttribute {
	pub signature: String
}

impl SignatureAttribute {
	pub fn parse(constant_pool: &ConstantPool, buf: Vec<u8>) -> Self {
		let index = buf.as_slice().read_u16::<BigEndian>().unwrap();
		let signature = constant_pool.utf8(index).unwrap().str.clone();
		SignatureAttribute {
			signature
		}
	}
	
	pub fn write<T: Seek + Write>(&self, wtr: &mut T, _constant_pool: &ConstantPool) {
		wtr.write_u16::<BigEndian>(0).unwrap(); // write name
		wtr.write_u32::<BigEndian>(2).unwrap(); // length
		wtr.write_u16::<BigEndian>(0).unwrap(); // cp ref
	}
}

#[derive(Clone, Debug, PartialEq)]
pub struct UnknownAttribute {
	pub name: String,
	pub buf: Vec<u8>
}

impl UnknownAttribute {
	pub fn parse(name: String, buf: Vec<u8>) -> Self {
		UnknownAttribute {
			name, buf
		}
	}
	
	pub fn write<T: Seek + Write>(&self, wtr: &mut T, _constant_pool: &ConstantPool) {
		wtr.write_u16::<BigEndian>(0).unwrap(); // write name
		wtr.write_u32::<BigEndian>(self.buf.len() as u32).unwrap(); // length
		wtr.write_all(self.buf.as_slice()).unwrap();
	}
}

#[derive(Clone, Debug, PartialEq)]
pub enum Attribute {
	ConstantValue(ConstantValueAttribute),
	Signature(SignatureAttribute),
	Code(CodeAttribute),
	Unknown(UnknownAttribute)
}

impl Attribute {
	pub fn parse<R: Read>(rdr: &mut R, source: &AttributeSource, version: &ClassVersion, constant_pool: &ConstantPool) -> Attribute {
		let name = constant_pool.utf8(rdr.read_u16::<BigEndian>().unwrap()).unwrap().str.clone();
		let attribute_length = rdr.read_u32::<BigEndian>().unwrap() as usize;
		let mut buf: Vec<u8> = Vec::with_capacity(attribute_length);
		rdr.take(attribute_length as u64).read_to_end(&mut buf).unwrap();
		let str = name.as_str();
		
		match source {
			AttributeSource::Class => {
				Attribute::Unknown(UnknownAttribute::parse(name, buf))
			},
			AttributeSource::Field => {
				if str == "ConstantValue" {
					Attribute::ConstantValue(ConstantValueAttribute::parse(constant_pool, buf))
				} else if str == "Signature" && version.major >= MajorVersion::JAVA_5 {
					Attribute::Signature(SignatureAttribute::parse(constant_pool, buf))
				} else {
					Attribute::Unknown(UnknownAttribute::parse(name, buf))
				}
			},
			AttributeSource::Method => {
				if str == "Code" {
					Attribute::Code(CodeAttribute::parse(version, constant_pool, buf))
				} else {
					Attribute::Unknown(UnknownAttribute::parse(name, buf))
				}
			}
			AttributeSource::Code => {
				Attribute::Unknown(UnknownAttribute::parse(name, buf))
			}
		}
	}
	
	pub fn write<T: Seek + Write>(&self, wtr: &mut T, constant_pool: &ConstantPool) {
		match self {
			Attribute::ConstantValue(t) => t.write(wtr, constant_pool),
			Attribute::Signature(t) => t.write(wtr, constant_pool),
			Attribute::Code(t) => t.write(wtr, constant_pool),
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
