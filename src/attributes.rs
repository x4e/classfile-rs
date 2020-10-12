use std::io::{Seek, Read, Write};
use crate::constantpool::{ConstantPool, ConstantType};
use byteorder::{ReadBytesExt, BigEndian};
use crate::version::{ClassVersion, MajorVersion};

#[derive(Clone, Debug, PartialEq)]
pub struct Attributes {
	attributes: Vec<Attribute>
}

impl Attributes {
	fn parse_generic<R: Seek + Read, F>(rdr: &mut R, constant_pool: &ConstantPool, attrib_parser: F) -> Attributes
		where F: Fn(String, Vec<u8>) -> Attribute {
		let num_attributes = rdr.read_u16::<BigEndian>().unwrap() as usize;
		let mut attributes: Vec<Attribute> = Vec::with_capacity(num_attributes);
		for _ in 0..num_attributes {
			let name = constant_pool.utf8(rdr.read_u16::<BigEndian>().unwrap()).unwrap().str.clone();
			let attribute_length = rdr.read_u32::<BigEndian>().unwrap() as usize;
			let mut buf: Vec<u8> = Vec::with_capacity(attribute_length);
			rdr.take(attribute_length as u64).read_to_end(&mut buf);
			attributes.push(attrib_parser(name, buf));
		}
		
		Attributes {
			attributes
		}
	}
	
	pub fn parse_field<R: Seek + Read>(rdr: &mut R, version: &ClassVersion, constant_pool: &ConstantPool) -> Self {
		Attributes::parse_generic(rdr, constant_pool, |name, buf| {
			let str = name.as_str();
			if str == "ConstantValue" {
				Attribute::ConstantValue(ConstantValueAttribute::parse(constant_pool, buf))
			} else if str == "Signature" && version.major >= MajorVersion::JAVA_5 {
				Attribute::Signature(SignatureAttribute::parse(constant_pool, buf))
			} else {
				Attribute::Unknown(UnknownAttribute { name, buf })
			}
		})
	}
	
	pub fn write<W: Seek + Write>(&self, _wtr: &mut W) {
		unimplemented!()
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
			ConstantType::Long(x) => ConstantValue::Long(x.bytes),
			ConstantType::Float(x) => ConstantValue::Float(x.bytes),
			ConstantType::Double(x) => ConstantValue::Double(x.bytes),
			ConstantType::Integer(x) => ConstantValue::Int(x.bytes),
			ConstantType::String(x) => ConstantValue::String(constant_pool.utf8(x.string_index).unwrap().str.clone()),
			x => panic!("Invalid constant value type {:#?}", x)
		};
		ConstantValueAttribute {
			value
		}
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
}

#[derive(Clone, Debug, PartialEq)]
pub struct UnknownAttribute {
	pub name: String,
	pub buf: Vec<u8>
}

#[derive(Clone, Debug, PartialEq)]
pub enum Attribute {
	ConstantValue(ConstantValueAttribute),
	Signature(SignatureAttribute),
	Unknown(UnknownAttribute)
}
