use crate::Serializable;
use std::io::{Read, Write, Seek};
use byteorder::{ReadBytesExt, BigEndian, WriteBytesExt};
use std::borrow::Borrow;
use derive_more::Constructor;
use crate::error::{Result, ParserError};
use enum_display_derive::DisplayDebug;
use std::fmt::{Debug, Formatter};
use std::collections::HashSet;
use linked_hash_map::LinkedHashMap;

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
			    x,
				index as usize
			)),
		}
	}
	
	pub fn fieldref(&self, index: CPIndex) -> Result<&FieldRefInfo> {
		match self.get(index)? {
			ConstantType::Fieldref(t) => Ok(t),
			x => Err(ParserError::incomp_cp(
				"FieldRef",
				x,
				index as usize
			)),
		}
	}
	
	pub fn any_method(&self, index: CPIndex) -> Result<(String, String, String, bool)> {
		match self.get(index)? {
			ConstantType::Methodref(method) => {
				let name_and_type = self.nameandtype(method.name_and_type_index)?;
				let class = self.utf8(self.class(method.class_index)?.name_index)?.str.clone();
				let name = self.utf8(name_and_type.name_index)?.str.clone();
				let descriptor = self.utf8(name_and_type.descriptor_index)?.str.clone();
				Ok((class, name, descriptor, false))
			},
			ConstantType::InterfaceMethodref(method) => {
				let name_and_type = self.nameandtype(method.name_and_type_index)?;
				let class = self.utf8(self.class(method.class_index)?.name_index)?.str.clone();
				let name = self.utf8(name_and_type.name_index)?.str.clone();
				let descriptor = self.utf8(name_and_type.descriptor_index)?.str.clone();
				Ok((class, name, descriptor, true))
			},
			x => Err(ParserError::incomp_cp(
				"AnyMethodRef",
				x,
				index as usize
			)),
		}
	}
	
	pub fn methodref(&self, index: CPIndex) -> Result<&MethodRefInfo> {
		match self.get(index)? {
			ConstantType::Methodref(t) => Ok(t),
			x => Err(ParserError::incomp_cp(
				"MethodRef",
				x,
				index as usize
			)),
		}
	}
	
	pub fn interfacemethodref(&self, index: CPIndex) -> Result<&InterfaceMethodRefInfo> {
		match self.get(index)? {
			ConstantType::InterfaceMethodref(t) => Ok(t),
			x => Err(ParserError::incomp_cp(
				"InterfaceMethodRef",
				x,
				index as usize
			)),
		}
	}
	
	pub fn string(&self, index: CPIndex) -> Result<&StringInfo> {
		match self.get(index)? {
			ConstantType::String(t) => Ok(t),
			x => Err(ParserError::incomp_cp(
				"String",
				x,
				index as usize
			)),
		}
	}
	
	pub fn integer(&self, index: CPIndex) -> Result<&IntegerInfo> {
		match self.get(index)? {
			ConstantType::Integer(t) => Ok(t),
			x => Err(ParserError::incomp_cp(
				"Integer",
				x,
				index as usize
			)),
		}
	}
	
	pub fn float(&self, index: CPIndex) -> Result<&FloatInfo> {
		match self.get(index)? {
			ConstantType::Float(t) => Ok(t),
			x => Err(ParserError::incomp_cp(
				"Float",
				x,
				index as usize
			)),
		}
	}
	
	pub fn long(&self, index: CPIndex) -> Result<&LongInfo> {
		match self.get(index)? {
			ConstantType::Long(t) => Ok(t),
			x => Err(ParserError::incomp_cp(
				"Long",
				x,
				index as usize
			)),
		}
	}
	
	pub fn double(&self, index: CPIndex) -> Result<&DoubleInfo> {
		match self.get(index)? {
			ConstantType::Double(t) => Ok(t),
			x => Err(ParserError::incomp_cp(
				"Double",
				x,
				index as usize
			)),
		}
	}
	
	pub fn nameandtype(&self, index: CPIndex) -> Result<&NameAndTypeInfo> {
		match self.get(index)? {
			ConstantType::NameAndType(t) => Ok(t),
			x => Err(ParserError::incomp_cp(
				"NameAndType",
				x,
				index as usize
			)),
		}
	}
	
	pub fn utf8(&self, index: CPIndex) -> Result<&Utf8Info> {
		match self.get(index)? {
			ConstantType::Utf8(t) => Ok(t),
			x => Err(ParserError::incomp_cp(
				"Utf8",
				x,
				index as usize
			)),
		}
	}
	
	pub fn methodhandle(&self, index: CPIndex) -> Result<&MethodHandleInfo> {
		match self.get(index)? {
			ConstantType::MethodHandle(t) => Ok(t),
			x => Err(ParserError::incomp_cp(
				"MethodHandle",
				x,
				index as usize
			)),
		}
	}
	
	pub fn methodtype(&self, index: CPIndex) -> Result<&MethodTypeInfo> {
		match self.get(index)? {
			ConstantType::MethodType(t) => Ok(t),
			x => Err(ParserError::incomp_cp(
				"MethodType",
				x,
				index as usize
			)),
		}
	}
	
	pub fn dynamicinfo(&self, index: CPIndex) -> Result<&DynamicInfo> {
		match self.get(index)? {
			ConstantType::Dynamic(t) => Ok(t),
			x => Err(ParserError::incomp_cp(
				"Dynamic",
				x,
				index as usize
			)),
		}
	}
	
	pub fn invokedynamicinfo(&self, index: CPIndex) -> Result<&InvokeDynamicInfo> {
		match self.get(index)? {
			ConstantType::InvokeDynamic(t) => Ok(t),
			x => Err(ParserError::incomp_cp(
				"InvokeDynamic",
				x,
				index as usize
			)),
		}
	}
	
	pub fn module(&self, index: CPIndex) -> Result<&ModuleInfo> {
		match self.get(index)? {
			ConstantType::Module(t) => Ok(t),
			x => Err(ParserError::incomp_cp(
				"Module",
				x,
				index as usize
			)),
		}
	}
	
	pub fn package(&self, index: CPIndex) -> Result<&PackageInfo> {
		match self.get(index)? {
			ConstantType::Package(t) => Ok(t),
			x => Err(ParserError::incomp_cp(
				"Package",
				x,
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
			if constant.double_size() {
				skip = true;
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

#[derive(Constructor, Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct ClassInfo {
	pub name_index: CPIndex
}
#[derive(Constructor, Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct FieldRefInfo {
	pub class_index: CPIndex,
	pub name_and_type_index: CPIndex
}
#[derive(Constructor, Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct MethodRefInfo {
	pub class_index: CPIndex,
	pub name_and_type_index: CPIndex
}
#[derive(Constructor, Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct InterfaceMethodRefInfo {
	pub class_index: CPIndex,
	pub name_and_type_index: CPIndex
}
#[derive(Constructor, Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct StringInfo {
	pub string_index: CPIndex
}
#[derive(Constructor, Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct IntegerInfo {
	pub bytes: i32
}
#[derive(Constructor, Copy, Clone, Debug, PartialEq, Hash)]
pub struct FloatInfo {
	pub bytes: f32
}
#[derive(Constructor, Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct LongInfo {
	pub bytes: i64
}
#[derive(Constructor, Copy, Clone, Debug, PartialEq, Hash)]
pub struct DoubleInfo {
	pub bytes: f64
}

impl Eq for DoubleInfo {}

#[derive(Constructor, Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct NameAndTypeInfo {
	pub name_index: CPIndex,
	pub descriptor_index: CPIndex
}
#[derive(Constructor, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Utf8Info {
	pub str: String
}

#[derive(Constructor, Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct MethodHandleInfo {
	pub reference: MethodHandleKind
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
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


#[derive(Constructor, Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct MethodTypeInfo {
	pub descriptor_index: CPIndex
}
#[derive(Constructor, Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct DynamicInfo {
	pub bootstrap_method_attr_index: CPIndex,
	pub name_and_type_index: CPIndex
}
#[derive(Constructor, Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct InvokeDynamicInfo {
	pub bootstrap_method_attr_index: CPIndex,
	pub name_and_type_index: CPIndex
}
#[derive(Constructor, Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct ModuleInfo {
	pub name_index: CPIndex
}
#[derive(Constructor, Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct PackageInfo {
	pub name_index: CPIndex
}

#[derive(Clone, PartialEq, Eq, Hash, DisplayDebug)]
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
	const CONSTANT_Utf8: u8 = 1;
	const CONSTANT_Integer: u8 = 3;
	const CONSTANT_Float: u8 = 4;
	const CONSTANT_Long: u8 = 5;
	const CONSTANT_Double: u8 = 6;
	const CONSTANT_Class: u8 = 7;
	const CONSTANT_String: u8 = 8;
	const CONSTANT_Fieldref: u8 = 9;
	const CONSTANT_Methodref: u8 = 10;
	const CONSTANT_InterfaceMethodref: u8 = 11;
	const CONSTANT_NameAndType: u8 = 12;
	const CONSTANT_MethodHandle: u8 = 15;
	const CONSTANT_MethodType: u8 = 16;
	const CONSTANT_Dynamic: u8 = 17;
	const CONSTANT_InvokeDynamic: u8 = 18;
	const CONSTANT_Module: u8 = 19;
	const CONSTANT_Package: u8 = 20;
	
	pub fn parse<R: Seek + Read>(rdr: &mut R, constants: &ConstantPool) -> Result<Self> {
		let tag = rdr.read_u8()?;
		Ok(match tag {
			ConstantType::CONSTANT_Class => ConstantType::Class {
				0: ClassInfo {
					name_index: rdr.read_u16::<BigEndian>()?
				},
			},
			ConstantType::CONSTANT_Fieldref => ConstantType::Fieldref {
				0: FieldRefInfo {
					class_index: rdr.read_u16::<BigEndian>()?,
					name_and_type_index: rdr.read_u16::<BigEndian>()?
				},
			},
			ConstantType::CONSTANT_Methodref => ConstantType::Methodref {
				0: MethodRefInfo {
					class_index: rdr.read_u16::<BigEndian>()?,
					name_and_type_index: rdr.read_u16::<BigEndian>()?
				},
			},
			ConstantType::CONSTANT_InterfaceMethodref => ConstantType::InterfaceMethodref {
				0: InterfaceMethodRefInfo {
					class_index: rdr.read_u16::<BigEndian>()?,
					name_and_type_index: rdr.read_u16::<BigEndian>()?
				},
			},
			ConstantType::CONSTANT_String => ConstantType::String {
				0: StringInfo {
					string_index: rdr.read_u16::<BigEndian>()?
				},
			},
			ConstantType::CONSTANT_Integer => ConstantType::Integer {
				0: IntegerInfo {
					bytes: rdr.read_i32::<BigEndian>()?
				},
			},
			ConstantType::CONSTANT_Float => ConstantType::Float {
				0: FloatInfo {
					bytes: rdr.read_f32::<BigEndian>()?
				},
			},
			ConstantType::CONSTANT_Long => ConstantType::Long {
				0: LongInfo {
					bytes: rdr.read_i64::<BigEndian>()?
				},
			},
			ConstantType::CONSTANT_Double => ConstantType::Double {
				0: DoubleInfo {
					bytes: rdr.read_f64::<BigEndian>()?
				},
			},
			ConstantType::CONSTANT_NameAndType => ConstantType::NameAndType {
				0: NameAndTypeInfo {
					name_index: rdr.read_u16::<BigEndian>()?,
					descriptor_index: rdr.read_u16::<BigEndian>()?
				},
			},
			ConstantType::CONSTANT_Utf8 => {
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
			ConstantType::CONSTANT_MethodHandle => {
				let reference_kind = rdr.read_u8()?;
				let reference_index = rdr.read_u16::<BigEndian>()? as usize;
				let reference = constants.get(reference_index as CPIndex)?;
				let handle_kind = match reference_kind {
					1 => MethodHandleKind::GetField(
						if let ConstantType::Fieldref(x) = reference {
							x.clone()
						} else {
							return Err(ParserError::other(format!("Invalid method handle ref at index {}", reference_index)))
						}
					),
					2 => MethodHandleKind::GetStatic(
						if let ConstantType::Fieldref(x) = reference {
							x.clone()
						} else {
							return Err(ParserError::other(format!("Invalid method handle ref at index {}", reference_index)))
						}
					),
					3 => MethodHandleKind::PutField(
						if let ConstantType::Fieldref(x) = reference {
							x.clone()
						} else {
							return Err(ParserError::other(format!("Invalid method handle ref at index {}", reference_index)))
						}
					),
					4 => MethodHandleKind::PutStatic(
						if let ConstantType::Fieldref(x) = reference {
							x.clone()
						} else {
							return Err(ParserError::other(format!("Invalid method handle ref at index {}", reference_index)))
						}
					),
					5 => MethodHandleKind::InvokeVirtual(
						if let ConstantType::Methodref(x) = reference {
							x.clone()
						} else {
							return Err(ParserError::other(format!("Invalid method handle ref at index {}", reference_index)))
						}
					),
					x => return Err(ParserError::other(format!("Unknown method handle type {}", x)))
				};
				ConstantType::MethodHandle(MethodHandleInfo::new(handle_kind))
			},
			ConstantType::CONSTANT_MethodType => ConstantType::MethodType {
				0: MethodTypeInfo {
					descriptor_index: rdr.read_u16::<BigEndian>()?
				},
			},
			ConstantType::CONSTANT_Dynamic => ConstantType::Dynamic {
				0: DynamicInfo {
					bootstrap_method_attr_index: rdr.read_u16::<BigEndian>()?,
					name_and_type_index: rdr.read_u16::<BigEndian>()?
				},
			},
			ConstantType::CONSTANT_InvokeDynamic => ConstantType::InvokeDynamic {
				0: InvokeDynamicInfo {
					bootstrap_method_attr_index: rdr.read_u16::<BigEndian>()?,
					name_and_type_index: rdr.read_u16::<BigEndian>()?
				},
			},
			ConstantType::CONSTANT_Module => ConstantType::Module {
				0: ModuleInfo {
					name_index: rdr.read_u16::<BigEndian>()?
				},
			},
			ConstantType::CONSTANT_Package => ConstantType::Package {
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
	
	pub fn double_size(&self) -> bool {
		match self {
			ConstantType::Double(..) | ConstantType::Long(..) => true,
			_ => false
		}
	}
}

pub struct ConstantPoolWriter {
	inner: LinkedHashMap<ConstantType, u16>,
	index: u16
}

impl ConstantPoolWriter {
	pub fn new() -> ConstantPoolWriter {
		ConstantPoolWriter {
			inner: LinkedHashMap::with_capacity(5),
			index: 1
		}
	}
	
	pub fn put(&mut self, constant: ConstantType) -> CPIndex {
		match self.inner.get(con) {
			Some(x) => *x,
			None => {
				let this_index = self.index;
				self.inner.insert(constant, this_index);
				self.index += if constant.double_size() { 2	} else { 1 };
				this_index
			}
		} as CPIndex
	}
	
	pub fn len(&self) -> u16 {
		self.index
	}
	
	pub fn class(&mut self, name_index: CPIndex) -> CPIndex {
		self.put(ConstantType::Class(ClassInfo {
			name_index
		}))
	}
	
	pub fn fieldref(&mut self, class_index: CPIndex, name_and_type_index: CPIndex) -> CPIndex {
		self.put(ConstantType::Fieldref(FieldRefInfo {
			class_index,
			name_and_type_index
		}))
	}
	
	pub fn methodref(&mut self, class_index: CPIndex, name_and_type_index: CPIndex) -> CPIndex {
		self.put(ConstantType::Methodref(MethodRefInfo {
			class_index,
			name_and_type_index
		}))
	}
	
	pub fn interfacemethodref(&mut self, class_index: CPIndex, name_and_type_index: CPIndex) -> CPIndex {
		self.put(ConstantType::InterfaceMethodref(InterfaceMethodRefInfo {
			class_index,
			name_and_type_index
		}))
	}
	
	pub fn string(&mut self, string_index: CPIndex) -> CPIndex {
		self.put(ConstantType::String(StringInfo {
			string_index
		}))
	}
	
	pub fn integer(&mut self, bytes: i32) -> CPIndex {
		self.put(ConstantType::Integer(IntegerInfo {
			bytes
		}))
	}
	
	pub fn float(&mut self, bytes: f32) -> CPIndex {
		self.put(ConstantType::Float(FloatInfo {
			bytes
		}))
	}
	
	pub fn long(&mut self, bytes: i64) -> CPIndex {
		self.put(ConstantType::Long(LongInfo {
			bytes
		}))
	}
	
	pub fn double(&mut self, bytes: f64) -> CPIndex {
		self.put(ConstantType::Double(DoubleInfo {
			bytes
		}))
	}
	
	pub fn nameandtype(&mut self, name_index: CPIndex, descriptor_index: CPIndex) -> CPIndex {
		self.put(ConstantType::NameAndType(NameAndTypeInfo {
			name_index,
			descriptor_index
		}))
	}
	
	pub fn utf8<T: Into<String>>(&mut self, str: T) -> CPIndex {
		self.put(ConstantType::Utf8(Utf8Info {
			str: str.into()
		}))
	}
	
	pub fn methodhandle(&mut self, reference: MethodHandleKind) -> CPIndex {
		self.put(ConstantType::MethodHandle(MethodHandleInfo {
			reference
		}))
	}
	
	pub fn methodtype(&mut self, descriptor_index: CPIndex) -> CPIndex {
		self.put(ConstantType::MethodType(MethodTypeInfo {
			descriptor_index
		}))
	}
	
	pub fn dynamicinfo(&mut self, bootstrap_method_attr_index: CPIndex, name_and_type_index: CPIndex) -> CPIndex {
		self.put(ConstantType::Dynamic(DynamicInfo {
			bootstrap_method_attr_index,
			name_and_type_index
		}))
	}
	
	pub fn invokedynamicinfo(&mut self, bootstrap_method_attr_index: CPIndex, name_and_type_index: CPIndex) -> CPIndex {
		self.put(ConstantType::InvokeDynamic(InvokeDynamicInfo {
			bootstrap_method_attr_index,
			name_and_type_index
		}))
	}
	
	pub fn module(&mut self, name_index: CPIndex) -> CPIndex {
		self.put(ConstantType::Module(ModuleInfo {
			name_index
		}))
	}
	
	pub fn package(&mut self, name_index: CPIndex) -> CPIndex {
		self.put(ConstantType::Package(PackageInfo {
			name_index
		}))
	}
}
