use crate::constantpool::{ConstantPool, ConstantType, ConstantPoolWriter};
use crate::version::{MajorVersion, ClassVersion};
use crate::code::CodeAttribute;
use crate::error::{Result, ParserError};
use byteorder::{ReadBytesExt, BigEndian, WriteBytesExt};
use std::io::{Write, Read, Cursor};
use derive_more::Constructor;
use crate::ast::LabelInsn;
use crate::utils::{ReadUtils, MapUtils};
use std::collections::HashMap;

#[allow(non_snake_case)]
pub mod Attributes {
	use std::io::{Read, Write};
	use crate::constantpool::{ConstantPool, ConstantPoolWriter};
	use byteorder::{ReadBytesExt, BigEndian, WriteBytesExt};
	use crate::version::{ClassVersion};
	use crate::attributes::{Attribute, AttributeSource};
	use std::collections::HashMap;
	use crate::ast::LabelInsn;
	
	pub fn parse<R: Read>(rdr: &mut R, source: AttributeSource, version: &ClassVersion, constant_pool: &ConstantPool, pc_label_map: &mut Option<HashMap<u32, LabelInsn>>) -> crate::Result<Vec<Attribute>> {
		let num_attributes = rdr.read_u16::<BigEndian>()? as usize;
		let mut attributes: Vec<Attribute> = Vec::with_capacity(num_attributes);
		for _ in 0..num_attributes {
			attributes.push(Attribute::parse(rdr, &source, version, constant_pool, pc_label_map.as_mut())?);
		}
		Ok(attributes)
	}
	
	pub fn write<W: Write>(wtr: &mut W, attributes: &Vec<Attribute>, constant_pool: &mut ConstantPoolWriter, label_pc_map: Option<&HashMap<LabelInsn, u32>>) -> crate::Result<()> {
		wtr.write_u16::<BigEndian>(attributes.len() as u16)?;
		for attribute in attributes.iter() {
			attribute.write(wtr, constant_pool, &label_pc_map)?;
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
			ConstantType::Long(x) => ConstantValue::Long(x.inner()),
			ConstantType::Float(x) => ConstantValue::Float(x.inner()),
			ConstantType::Double(x) => ConstantValue::Double(x.inner()),
			ConstantType::Integer(x) => ConstantValue::Int(x.inner()),
			ConstantType::String(x) => ConstantValue::String(constant_pool.utf8(x.utf_index)?.str.clone()),
			x => panic!("Invalid constant value type {:#?} at index {}", x, index)
		};
		Ok(ConstantValueAttribute {
			value
		})
	}
	
	pub fn write<T: Write>(&self, wtr: &mut T, constant_pool: &mut ConstantPoolWriter) -> Result<()> {
		let const_ref = match self.value.clone() {
			ConstantValue::Long(x) => constant_pool.long(x),
			ConstantValue::Float(x) => constant_pool.float(x),
			ConstantValue::Double(x) => constant_pool.double(x),
			ConstantValue::Int(x) => constant_pool.integer(x),
			ConstantValue::String(x) => {
				let utf = constant_pool.utf8(x);
				constant_pool.string(utf)
			}
		};
		wtr.write_u16::<BigEndian>(const_ref)?; // cp ref
		Ok(())
	}
}

#[derive(Clone, Debug, PartialEq)]
pub struct SignatureAttribute {
	pub signature: String
}

impl SignatureAttribute {
	pub fn new(signature: String) -> Self {
		SignatureAttribute {
			signature
		}
	}
	
	pub fn parse(constant_pool: &ConstantPool, buf: Vec<u8>) -> Result<Self> {
		let index = buf.as_slice().read_u16::<BigEndian>()?;
		let signature = constant_pool.utf8(index)?.str.clone();
		Ok(SignatureAttribute {
			signature
		})
	}
	
	pub fn write<T: Write>(&self, wtr: &mut T, constant_pool: &mut ConstantPoolWriter) -> Result<()> {
		wtr.write_u16::<BigEndian>(constant_pool.utf8(self.signature.clone()))?; // cp ref
		Ok(())
	}
}

#[derive(Clone, Debug, PartialEq)]
pub struct ExceptionsAttribute {
	pub exceptions: Vec<String>
}

impl ExceptionsAttribute {
	pub fn new(exceptions: Vec<String>) -> Self {
		ExceptionsAttribute {
			exceptions
		}
	}
	
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
	
	pub fn write<T: Write>(&self, wtr: &mut T, constant_pool: &mut ConstantPoolWriter) -> Result<()> {
		let num_exceptions = self.exceptions.len();
		wtr.write_u16::<BigEndian>(num_exceptions as u16)?;
		for exception in self.exceptions.iter() {
			wtr.write_u16::<BigEndian>(constant_pool.utf8(exception.clone()))?;
		}
		Ok(())
	}
}

#[derive(Constructor, Clone, Debug, PartialEq)]
pub struct UnknownAttribute {
	pub name: String,
	pub buf: Vec<u8>
}

impl UnknownAttribute {
	pub fn parse(name: String, buf: Vec<u8>) -> Result<Self> {
		Ok(UnknownAttribute::new(name, buf))
	}
	
	pub fn write<T: Write>(&self, wtr: &mut T, _constant_pool: &mut ConstantPoolWriter) -> Result<()> {
		wtr.write_all(self.buf.as_slice())?;
		Ok(())
	}
	
	pub fn len(&self) -> usize {
		self.buf.len()
	}
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
	
	pub fn write<T: Write>(&self, wtr: &mut T, constant_pool: &mut ConstantPoolWriter) -> Result<()> {
		wtr.write_u16::<BigEndian>(constant_pool.utf8(self.source_file.clone()))?;
		Ok(())
	}
}

#[derive(Clone, Debug, PartialEq)]
pub struct LocalVariableTableAttribute {
	pub variables: Vec<LocalVariable>
}

#[derive(Clone, Debug, PartialEq)]
pub struct LocalVariable {
	pub start: LabelInsn,
	pub end: LabelInsn,
	pub name: String,
	pub descriptor: String,
	pub index: u16
}

impl LocalVariableTableAttribute {
	pub fn parse(constant_pool: &ConstantPool, buf: Vec<u8>, pc_label_map: &mut HashMap<u32, LabelInsn>) -> Result<Self> {
		let mut buf = Cursor::new(buf);
		let num_vars = buf.read_u16::<BigEndian>()? as usize;
		let mut variables: Vec<LocalVariable> = Vec::with_capacity(num_vars);
		for _ in 0..num_vars {
			variables.push(LocalVariable::parse(constant_pool, &mut buf, pc_label_map)?)
		}
		Ok(LocalVariableTableAttribute {
			variables
		})
	}
	
	pub fn write<T: Write>(&self, wtr: &mut T, constant_pool: &mut ConstantPoolWriter, label_pc_map: &HashMap<LabelInsn, u32>) -> Result<()> {
		wtr.write_u16::<BigEndian>(self.variables.len() as u16)?;
		for var in self.variables.iter() {
			var.write(wtr, constant_pool, label_pc_map)?;
		}
		Ok(())
	}
}

impl LocalVariable {
	pub fn parse(constant_pool: &ConstantPool, buf: &mut Cursor<Vec<u8>>, pc_label_map: &mut HashMap<u32, LabelInsn>) -> Result<Self> {
		let start_pc = buf.read_u16::<BigEndian>()? as u32;
		let end_pc = start_pc + (buf.read_u16::<BigEndian>()? as u32);
		pc_label_map.insert_if_not_present(start_pc, LabelInsn::new(pc_label_map.len() as u32));
		pc_label_map.insert_if_not_present(end_pc, LabelInsn::new(pc_label_map.len() as u32));
		
		let name = constant_pool.utf8_inner(buf.read_u16::<BigEndian>()?)?;
		let descriptor = constant_pool.utf8_inner(buf.read_u16::<BigEndian>()?)?;
		let index = buf.read_u16::<BigEndian>()?;
		
		Ok(LocalVariable {
			start: *pc_label_map.get(&start_pc).ok_or_else(ParserError::unmapped_label)?,
			end: *pc_label_map.get(&end_pc).ok_or_else(ParserError::unmapped_label)?,
			name,
			descriptor,
			index
		})
	}
	
	pub fn write<T: Write>(&self, wtr: &mut T, constant_pool: &mut ConstantPoolWriter, label_pc_map: &HashMap<LabelInsn, u32>) -> Result<()> {
		let start_pc = *label_pc_map.get(&self.start).ok_or_else(ParserError::unmapped_label)?;
		wtr.write_u16::<BigEndian>(start_pc as u16)?;
		let end_pc = *label_pc_map.get(&self.end).ok_or_else(ParserError::unmapped_label)?;
		wtr.write_u16::<BigEndian>((end_pc - start_pc) as u16)?;
		ZIL
		wtr.write_u16::<BigEndian>(constant_pool.utf8(self.name.clone()))?;
		wtr.write_u16::<BigEndian>(constant_pool.utf8(self.descriptor.clone()))?;
		
		wtr.write_u16::<BigEndian>(self.index)?;
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
	LocalVariableTable(LocalVariableTableAttribute),
	Unknown(UnknownAttribute)
}

impl Attribute {
	pub fn parse<R: Read>(rdr: &mut R, source: &AttributeSource, version: &ClassVersion, constant_pool: &ConstantPool, pc_label_map: Option<&mut HashMap<u32, LabelInsn>>) -> Result<Attribute> {
		let name = constant_pool.utf8(rdr.read_u16::<BigEndian>()?)?.str.clone();
		let attribute_length = rdr.read_u32::<BigEndian>()? as usize;
		let buf: Vec<u8> = rdr.read_nbytes(attribute_length as usize)?;
		let str = name.as_str();
		
		let attr = match source {
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
				let pc_label_map = pc_label_map.unwrap();
				if str == "LocalVariableTable" {
					Attribute::LocalVariableTable(LocalVariableTableAttribute::parse(constant_pool, buf, pc_label_map)?)
				//} else if str == "LocalVariableTypeTable" && version.major >= MajorVersion::JAVA_5 {
				
				} else {
					Attribute::Unknown(UnknownAttribute::parse(name, buf)?)
				}
			}
		};
		Ok(attr)
	}
	
	pub fn write<T: Write>(&self, wtr: &mut T, constant_pool: &mut ConstantPoolWriter, label_pc_map: &Option<&HashMap<LabelInsn, u32>>) -> Result<()> {
		match self {
			Attribute::ConstantValue(t) => {
				let mut buf: Vec<u8> = Vec::new();
				wtr.write_u16::<BigEndian>(constant_pool.utf8("ConstantValue"))?;
				t.write(&mut buf, constant_pool)?;
				wtr.write_u32::<BigEndian>(buf.len() as u32)?;
				wtr.write(buf.as_slice())?;
			},
			Attribute::Signature(t) => {
				let mut buf: Vec<u8> = Vec::new();
				wtr.write_u16::<BigEndian>(constant_pool.utf8("Signature"))?;
				t.write(&mut buf, constant_pool)?;
				wtr.write_u32::<BigEndian>(buf.len() as u32)?;
				wtr.write(buf.as_slice())?;
			},
			Attribute::Code(t) => {
				let mut buf: Vec<u8> = Vec::new();
				wtr.write_u16::<BigEndian>(constant_pool.utf8("Code"))?;
				t.write(&mut buf, constant_pool)?;
				wtr.write_u32::<BigEndian>(buf.len() as u32)?;
				wtr.write(buf.as_slice())?;
			},
			Attribute::Exceptions(t) => {
				let mut buf: Vec<u8> = Vec::new();
				wtr.write_u16::<BigEndian>(constant_pool.utf8("Exceptions"))?;
				t.write(&mut buf, constant_pool)?;
				wtr.write_u32::<BigEndian>(buf.len() as u32)?;
				wtr.write(buf.as_slice())?;
			},
			Attribute::SourceFile(t) => {
				let mut buf: Vec<u8> = Vec::new();
				wtr.write_u16::<BigEndian>(constant_pool.utf8("SourceFile"))?;
				t.write(&mut buf, constant_pool)?;
				wtr.write_u32::<BigEndian>(buf.len() as u32)?;
				wtr.write(buf.as_slice())?;
			},
			Attribute::LocalVariableTable(t) => {
				let label_pc_map = label_pc_map.unwrap();
				let mut buf: Vec<u8> = Vec::new();
				wtr.write_u16::<BigEndian>(constant_pool.utf8("LocalVariableTable"))?;
				t.write(&mut buf, constant_pool, label_pc_map)?;
				wtr.write_u32::<BigEndian>(buf.len() as u32)?;
				wtr.write(buf.as_slice())?;
			},
			Attribute::Unknown(t) => {
				wtr.write_u16::<BigEndian>(constant_pool.utf8(t.name.clone()))?;
				wtr.write_u32::<BigEndian>(t.len() as u32)?;
				t.write(wtr, constant_pool)?;
			}
		};
		Ok(())
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
