use crate::Serializable;
use std::io::{Read, Write, Seek};
use byteorder::{ReadBytesExt, BigEndian, WriteBytesExt};
use std::borrow::Borrow;
use derive_more::Constructor;
use crate::error::{Result, ParserError};
use enum_display_derive::DisplayDebug;
use std::fmt::{Debug, Formatter};

pub type CPIndex = u16;

#[derive(Clone, PartialEq)]
pub struct ConstantPool {
	inner: Vec<Option<ConstantType>>
}

impl Debug for ConstantPool {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		let mut list = f.debug_list();
		for x in self.inner.iter() {
			match x {
				Some(x) => list.entry(x),
				None => list.entry(x)
			};
		}
		list.finish()
	}
}

#[allow(dead_code)]
impl ConstantPool {
	pub fn new() -> ConstantPool {
		ConstantPool {
			inner: Vec::with_capacity(12)
		}
	}
	
	pub fn get(&self, index: CPIndex) -> Result<&ConstantType> {
		match self.inner.get(index as usize) {
			Some(Some(x)) => {
				Ok(x)
			}
			_ => Err(ParserError::bad_cp_index(index))
		}
	}
	
	pub fn set(&mut self, index: CPIndex, value: Option<ConstantType>) {
		let index = index as usize;
		if index > self.inner.len() - 1 {
			self.inner.resize(index + 1, None);
		}
		self.inner[index] = value
	}
	
	pub fn class(&self, index: CPIndex) -> Result<&ClassInfo> {
		match self.get(index)? {
			ConstantType::Class(t) => Ok(t),
			x => Err(ParserError::incomp_cp(
				"Class",
			    format!("{:#?}", x),
				index as usize
			)),
		}
	}
	
	pub fn fieldref(&self, index: CPIndex) -> Result<&FieldRefInfo> {
		match self.get(index)? {
			ConstantType::Fieldref(t) => Ok(t),
			x => Err(ParserError::incomp_cp(
				"FieldRef",
				format!("{:#?}", x),
				index as usize
			)),
		}
	}
	
	pub fn methodref(&self, index: CPIndex) -> Result<&MethodRefInfo> {
		match self.get(index)? {
			ConstantType::Methodref(t) => Ok(t),
			x => Err(ParserError::incomp_cp(
				"MethodRef",
				format!("{:#?}", x),
				index as usize
			)),
		}
	}
	
	pub fn interfacemethodref(&self, index: CPIndex) -> Result<&InterfaceMethodRefInfo> {
		match self.get(index)? {
			ConstantType::InterfaceMethodref(t) => Ok(t),
			x => Err(ParserError::incomp_cp(
				"InterfaceMethodRef",
				format!("{:#?}", x),
				index as usize
			)),
		}
	}
	
	pub fn string(&self, index: CPIndex) -> Result<&StringInfo> {
		match self.get(index)? {
			ConstantType::String(t) => Ok(t),
			x => Err(ParserError::incomp_cp(
				"String",
				format!("{:#?}", x),
				index as usize
			)),
		}
	}
	
	pub fn integer(&self, index: CPIndex) -> Result<&IntegerInfo> {
		match self.get(index)? {
			ConstantType::Integer(t) => Ok(t),
			x => Err(ParserError::incomp_cp(
				"Integer",
				format!("{:#?}", x),
				index as usize
			)),
		}
	}
	
	pub fn float(&self, index: CPIndex) -> Result<&FloatInfo> {
		match self.get(index)? {
			ConstantType::Float(t) => Ok(t),
			x => Err(ParserError::incomp_cp(
				"Float",
				format!("{:#?}", x),
				index as usize
			)),
		}
	}
	
	pub fn long(&self, index: CPIndex) -> Result<&LongInfo> {
		match self.get(index)? {
			ConstantType::Long(t) => Ok(t),
			x => Err(ParserError::incomp_cp(
				"Long",
				format!("{:#?}", x),
				index as usize
			)),
		}
	}
	
	pub fn double(&self, index: CPIndex) -> Result<&DoubleInfo> {
		match self.get(index)? {
			ConstantType::Double(t) => Ok(t),
			x => Err(ParserError::incomp_cp(
				"Double",
				format!("{:#?}", x),
				index as usize
			)),
		}
	}
	
	pub fn nameandtype(&self, index: CPIndex) -> Result<&NameAndTypeInfo> {
		match self.get(index)? {
			ConstantType::NameAndType(t) => Ok(t),
			x => Err(ParserError::incomp_cp(
				"NameAndType",
				format!("{:#?}", x),
				index as usize
			)),
		}
	}
	
	pub fn utf8(&self, index: CPIndex) -> Result<&Utf8Info> {
		match self.get(index)? {
			ConstantType::Utf8(t) => Ok(t),
			x => Err(ParserError::incomp_cp(
				"Utf8",
				format!("{:#?}", x),
				index as usize
			)),
		}
	}
	
	pub fn methodhandle(&self, index: CPIndex) -> Result<&MethodHandleInfo> {
		match self.get(index)? {
			ConstantType::MethodHandle(t) => Ok(t),
			x => Err(ParserError::incomp_cp(
				"MethodHandle",
				format!("{:#?}", x),
				index as usize
			)),
		}
	}
	
	pub fn methodtype(&self, index: CPIndex) -> Result<&MethodTypeInfo> {
		match self.get(index)? {
			ConstantType::MethodType(t) => Ok(t),
			x => Err(ParserError::incomp_cp(
				"MethodType",
				format!("{:#?}", x),
				index as usize
			)),
		}
	}
	
	pub fn dynamicinfo(&self, index: CPIndex) -> Result<&DynamicInfo> {
		match self.get(index)? {
			ConstantType::Dynamic(t) => Ok(t),
			x => Err(ParserError::incomp_cp(
				"Dynamic",
				format!("{:#?}", x),
				index as usize
			)),
		}
	}
	
	pub fn invokedynamicinfo(&self, index: CPIndex) -> Result<&InvokeDynamicInfo> {
		match self.get(index)? {
			ConstantType::InvokeDynamic(t) => Ok(t),
			x => Err(ParserError::incomp_cp(
				"InvokeDynamic",
				format!("{:#?}", x),
				index as usize
			)),
		}
	}
	
	pub fn module(&self, index: CPIndex) -> Result<&ModuleInfo> {
		match self.get(index)? {
			ConstantType::Module(t) => Ok(t),
			x => Err(ParserError::incomp_cp(
				"Module",
				format!("{:#?}", x),
				index as usize
			)),
		}
	}
	
	pub fn package(&self, index: CPIndex) -> Result<&PackageInfo> {
		match self.get(index)? {
			ConstantType::Package(t) => Ok(t),
			x => Err(ParserError::incomp_cp(
				"Package",
				format!("{:#?}", x),
				index as usize
			)),
		}
	}
}

impl Serializable for ConstantPool {
	fn parse<R: Seek + Read>(rdr: &mut R) -> Result<Self> {
		let size = rdr.read_u16::<BigEndian>()? as usize;
		let mut cp = ConstantPool {
			inner: vec![None; size]
		};
		let mut skip = false;
		for i in 1..size {
			if skip {
				skip = false;
				continue
			}
			let constant = ConstantType::parse(rdr, &cp)?;
			match constant {
				ConstantType::Double(..) | ConstantType::Long(..) => {
					skip = true;
				}
				_ => {}
			}
			cp.set(i as CPIndex, Some(constant));
		}
		
		Ok(cp)
	}
	
	fn write<W: Seek + Write>(&self, wtr: &mut W) -> Result<()> {
		wtr.write_u16::<BigEndian>(self.inner.len() as u16)?;
		Ok(())
	}
}

#[derive(Constructor, Copy, Clone, Debug, PartialEq)]
pub struct ClassInfo {
	pub name_index: CPIndex
}
#[derive(Constructor, Copy, Clone, Debug, PartialEq)]
pub struct FieldRefInfo {
	pub class_index: CPIndex,
	pub name_and_type_index: CPIndex
}
#[derive(Constructor, Copy, Clone, Debug, PartialEq)]
pub struct MethodRefInfo {
	pub class_index: CPIndex,
	pub name_and_type_index: CPIndex
}
#[derive(Constructor, Copy, Clone, Debug, PartialEq)]
pub struct InterfaceMethodRefInfo {
	pub class_index: CPIndex,
	pub name_and_type_index: CPIndex
}
#[derive(Constructor, Copy, Clone, Debug, PartialEq)]
pub struct StringInfo {
	pub string_index: CPIndex
}
#[derive(Constructor, Copy, Clone, Debug, PartialEq)]
pub struct IntegerInfo {
	pub bytes: i32
}
#[derive(Constructor, Copy, Clone, Debug, PartialEq)]
pub struct FloatInfo {
	pub bytes: f32
}
#[derive(Constructor, Copy, Clone, Debug, PartialEq)]
pub struct LongInfo {
	pub bytes: i64
}
#[derive(Constructor, Copy, Clone, Debug, PartialEq)]
pub struct DoubleInfo {
	pub bytes: f64
}
#[derive(Constructor, Copy, Clone, Debug, PartialEq)]
pub struct NameAndTypeInfo {
	pub name_index: CPIndex,
	pub descriptor_index: CPIndex
}
#[derive(Constructor, Clone, Debug, PartialEq)]
pub struct Utf8Info {
	pub str: String
}

#[derive(Constructor, Copy, Clone, Debug, PartialEq)]
pub struct MethodHandleInfo {
	pub reference: MethodHandleKind
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum MethodHandleKind {
	GetField(FieldRefInfo),
	GetStatic(FieldRefInfo),
	PutField(FieldRefInfo),
	PutStatic(FieldRefInfo),
	InvokeVirtual(MethodRefInfo),
	NewInvokeSpecial(MethodRefInfo),
	InvokeStatic((Option<MethodRefInfo>, Option<InterfaceMethodRefInfo>)),
	InvokeSpecial((Option<MethodRefInfo>, Option<InterfaceMethodRefInfo>)),
}


#[derive(Constructor, Copy, Clone, Debug, PartialEq)]
pub struct MethodTypeInfo {
	pub descriptor_index: CPIndex
}
#[derive(Constructor, Copy, Clone, Debug, PartialEq)]
pub struct DynamicInfo {
	pub bootstrap_method_attr_index: CPIndex,
	pub name_and_type_index: CPIndex
}
#[derive(Constructor, Copy, Clone, Debug, PartialEq)]
pub struct InvokeDynamicInfo {
	pub bootstrap_method_attr_index: CPIndex,
	pub name_and_type_index: CPIndex
}
#[derive(Constructor, Copy, Clone, Debug, PartialEq)]
pub struct ModuleInfo {
	pub name_index: CPIndex
}
#[derive(Constructor, Copy, Clone, Debug, PartialEq)]
pub struct PackageInfo {
	pub name_index: CPIndex
}

#[derive(Clone, PartialEq, DisplayDebug)]
pub enum ConstantType {
	Class (ClassInfo),
	Fieldref (FieldRefInfo),
	Methodref (MethodRefInfo),
	InterfaceMethodref (InterfaceMethodRefInfo),
	String (StringInfo),
	Integer (IntegerInfo),
	Float (FloatInfo),
	Long (LongInfo),
	Double (DoubleInfo),
	NameAndType (NameAndTypeInfo),
	Utf8 (Utf8Info),
	MethodHandle (MethodHandleInfo),
	MethodType (MethodTypeInfo),
	Dynamic (DynamicInfo),
	InvokeDynamic (InvokeDynamicInfo),
	Module (ModuleInfo),
	Package (PackageInfo)
}

impl ConstantType {
	pub fn parse<R: Seek + Read>(rdr: &mut R, constants: &ConstantPool) -> Result<Self> {
		let tag = rdr.read_u8()?;
		Ok(match tag {
			7 => ConstantType::Class {
				0: ClassInfo {
					name_index: rdr.read_u16::<BigEndian>()?
				},
			},
			9 => ConstantType::Fieldref {
				0: FieldRefInfo {
					class_index: rdr.read_u16::<BigEndian>()?,
					name_and_type_index: rdr.read_u16::<BigEndian>()?
				},
			},
			10 => ConstantType::Methodref {
				0: MethodRefInfo {
					class_index: rdr.read_u16::<BigEndian>()?,
					name_and_type_index: rdr.read_u16::<BigEndian>()?
				},
			},
			11 => ConstantType::InterfaceMethodref {
				0: InterfaceMethodRefInfo {
					class_index: rdr.read_u16::<BigEndian>()?,
					name_and_type_index: rdr.read_u16::<BigEndian>()?
				},
			},
			8 => ConstantType::String {
				0: StringInfo {
					string_index: rdr.read_u16::<BigEndian>()?
				},
			},
			3 => ConstantType::Integer {
				0: IntegerInfo {
					bytes: rdr.read_i32::<BigEndian>()?
				},
			},
			4 => ConstantType::Float {
				0: FloatInfo {
					bytes: rdr.read_f32::<BigEndian>()?
				},
			},
			5 => ConstantType::Long {
				0: LongInfo {
					bytes: rdr.read_i64::<BigEndian>()?
				},
			},
			6 => ConstantType::Double {
				0: DoubleInfo {
					bytes: rdr.read_f64::<BigEndian>()?
				},
			},
			12 => ConstantType::NameAndType {
				0: NameAndTypeInfo {
					name_index: rdr.read_u16::<BigEndian>()?,
					descriptor_index: rdr.read_u16::<BigEndian>()?
				},
			},
			1 => {
				let length = rdr.read_u16::<BigEndian>()? as usize;
				let mut bytes: Vec<u8> = Vec::with_capacity(length);
				for _ in 0..length {
					bytes.push(rdr.read_u8()?);
				}
				let mut str = String::with_capacity(length);
				
				let mut valid = true;
				let mut i = 0usize;
				while i < length {
					let c1 = bytes[i];
					i += 1;
					if c1 & 0x80 == 0 {
						str.push(c1 as char);
					} else if c1 & 0xE0 == 0xC0 {
						let c2 = rdr.read_u8()?;
						i += 1;
						if c2 & 0xC0 == 0x80 {
							str.push((((c1 & 0x1F) << 6) + (c2 & 0x3F)) as char);
						} else {
							valid = false;
							break;
						}
					} else if c1 & 0xF0 == 0xE0 {
						let c2 = bytes[i];
						i += 1;
						let c3 = bytes[i];
						i += 1;
						if c2 & 0xC0 == 0x80 && c3 & 0xC0 == 0x80 {
							#[allow(arithmetic_overflow)]
							str.push((((c1 & 0xF) << 12) + ((c2 & 0x3F) << 6) + (c3 & 0x3F)) as char)
						} else {
							valid = false;
							break;
						}
					}
				}
				if !valid {
					str = String::from_utf8_lossy(bytes.borrow()).into_owned();//String::from(String::from_utf8_lossy(bytes.borrow()).borrow());
				}
				
				ConstantType::Utf8 {
					0: Utf8Info {
						str
					},
				}
			},
			15 => {
				let reference_kind = rdr.read_u8()?;
				let reference_index = rdr.read_u16::<BigEndian>()? as usize;
				let reference = constants.get(reference_index as CPIndex)?;
				let handle_kind = match reference_kind {
					1 => MethodHandleKind::GetField(
						if let ConstantType::Fieldref(x) = reference {
							x.clone()
						} else {
							panic!("Invalid method handle ref at index {}", reference_index)
						}
					),
					2 => MethodHandleKind::GetStatic(
						if let ConstantType::Fieldref(x) = reference {
							x.clone()
						} else {
							panic!("Invalid method handle ref at index {}", reference_index)
						}
					),
					3 => MethodHandleKind::PutField(
						if let ConstantType::Fieldref(x) = reference {
							x.clone()
						} else {
							panic!("Invalid method handle ref at index {}", reference_index)
						}
					),
					4 => MethodHandleKind::PutStatic(
						if let ConstantType::Fieldref(x) = reference {
							x.clone()
						} else {
							panic!("Invalid method handle ref at index {}", reference_index)
						}
					),
					5 => MethodHandleKind::InvokeVirtual(
						if let ConstantType::Methodref(x) = reference {
							x.clone()
						} else {
							panic!("Invalid method handle ref at index {}", reference_index)
						}
					),
					_ => panic!("Unknown method handle type {}", reference_kind)
				};
				ConstantType::MethodHandle(MethodHandleInfo::new(handle_kind))
			},
			16 => ConstantType::MethodType {
				0: MethodTypeInfo {
					descriptor_index: rdr.read_u16::<BigEndian>()?
				},
			},
			17 => ConstantType::Dynamic {
				0: DynamicInfo {
					bootstrap_method_attr_index: rdr.read_u16::<BigEndian>()?,
					name_and_type_index: rdr.read_u16::<BigEndian>()?
				},
			},
			18 => ConstantType::InvokeDynamic {
				0: InvokeDynamicInfo {
					bootstrap_method_attr_index: rdr.read_u16::<BigEndian>()?,
					name_and_type_index: rdr.read_u16::<BigEndian>()?
				},
			},
			19 => ConstantType::Module {
				0: ModuleInfo {
					name_index: rdr.read_u16::<BigEndian>()?
				},
			},
			20 => ConstantType::Package {
				0: PackageInfo {
					name_index: rdr.read_u16::<BigEndian>()?
				},
			},
			_ => return Err(ParserError::unrecognised("constant tag", tag.to_string()))
		})
	}
	
	pub fn write<W: Seek + Write>(&self, wtr: &mut W) -> Result<()> {
		match self {
			ConstantType::Class { .. } => {
				wtr.write_u8(7)?
			},
			_ => return Err(ParserError::unimplemented("Constant Pool Writing"))
		}
		Ok(())
	}
}
