use crate::Serializable;
use crate::utils::ReadUtils;
use crate::error::{Result, ParserError};
use std::io::{Read, Write};
use byteorder::{ReadBytesExt, BigEndian, WriteBytesExt};
use std::borrow::{Cow};
use derive_more::Constructor;
use enum_display_derive::DisplayDebug;
use std::fmt::{Debug, Formatter};
use linked_hash_map::LinkedHashMap;
use std::hash::{Hash};

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
	
	pub fn any_method(&self, index: CPIndex) -> Result<(&MethodRefInfo, bool)> {
		match self.get(index)? {
			ConstantType::Methodref(method) => Ok((method, false)),
			ConstantType::InterfaceMethodref(method) => Ok((method, true)),
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
	
	pub fn interfacemethodref(&self, index: CPIndex) -> Result<&MethodRefInfo> {
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
	fn parse<R: Read>(rdr: &mut R) -> Result<Self> {
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
			let constant = ConstantType::parse(rdr)?;
			if constant.double_size() {
				skip = true;
			}
			cp.set(i as CPIndex, Some(constant));
		}
		
		Ok(cp)
	}
	
	fn write<W: Write>(&self, wtr: &mut W) -> Result<()> {
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
pub struct StringInfo {
	pub utf_index: CPIndex
}

#[derive(Constructor, Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct IntegerInfo {
	inner: i32
}
impl IntegerInfo {
	pub fn inner(&self) -> i32 {
		self.inner
	}
}

#[derive(Constructor, Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct LongInfo {
	inner: i64
}
impl LongInfo {
	pub fn inner(&self) -> i64 {
		self.inner
	}
}

/// Rust floats do not support Eq and Hash
/// This is because its just too hard to correctly compare floats
/// For our purpose however we dont care too much about equality and more about not (?) equality
/// In the end if two of the same floats are not compared equal to each other then we just make
/// two constant pool entries and who cares
/// Because of this we will store the float as an integer and let rust do integer comparisons on it
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct FloatInfo {
	inner: u32
}
impl FloatInfo {
	pub fn new(inner: f32) -> Self {
		FloatInfo {
			inner: inner.to_bits()
		}
	}
	pub fn inner(&self) -> f32 {
		f32::from_bits(self.inner)
	}
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct DoubleInfo {
	inner: u64
}
impl DoubleInfo {
	pub fn new(inner: f64) -> Self {
		DoubleInfo {
			inner: inner.to_bits()
		}
	}
	pub fn inner(&self) -> f64 {
		f64::from_bits(self.inner)
	}
}

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
	pub kind: MethodHandleKind,
	pub reference: CPIndex
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum MethodHandleKind {
	GetField,
	GetStatic,
	PutField,
	PutStatic,
	InvokeVirtual,
	InvokeStatic,
	InvokeSpecial,
	NewInvokeSpecial,
	InvokeInterface
}

#[allow(non_upper_case_globals)]
impl MethodHandleKind {
	const REF_getField: u8 = 1;
	const REF_getStatic: u8 = 2;
	const REF_putField: u8 = 3;
	const REF_putStatic: u8 = 4;
	const REF_invokeVirtual: u8 = 5;
	const REF_invokeStatic: u8 = 6;
	const REF_invokeSpecial: u8 = 7;
	const REF_newInvokeSpecial: u8 = 8;
	const REF_invokeInterface: u8 = 9;
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
	InterfaceMethodref (MethodRefInfo),
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

#[allow(non_upper_case_globals)]
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
	
	pub fn parse<R: Read>(rdr: &mut R) -> Result<Self> {
		let tag = rdr.read_u8()?;
		Ok(match tag {
			ConstantType::CONSTANT_Class => ConstantType::Class (
				ClassInfo {
					name_index: rdr.read_u16::<BigEndian>()?
				},
			),
			ConstantType::CONSTANT_Fieldref => ConstantType::Fieldref (
				FieldRefInfo {
					class_index: rdr.read_u16::<BigEndian>()?,
					name_and_type_index: rdr.read_u16::<BigEndian>()?
				},
			),
			ConstantType::CONSTANT_Methodref => ConstantType::Methodref (
				MethodRefInfo {
					class_index: rdr.read_u16::<BigEndian>()?,
					name_and_type_index: rdr.read_u16::<BigEndian>()?
				},
			),
			ConstantType::CONSTANT_InterfaceMethodref => ConstantType::InterfaceMethodref (
				MethodRefInfo {
					class_index: rdr.read_u16::<BigEndian>()?,
					name_and_type_index: rdr.read_u16::<BigEndian>()?
				},
			),
			ConstantType::CONSTANT_String => ConstantType::String (
				StringInfo {
					utf_index: rdr.read_u16::<BigEndian>()?
				},
			),
			ConstantType::CONSTANT_Integer => ConstantType::Integer (
				IntegerInfo::new(rdr.read_i32::<BigEndian>()?),
			),
			ConstantType::CONSTANT_Float => ConstantType::Float (
				FloatInfo::new(rdr.read_f32::<BigEndian>()?),
			),
			ConstantType::CONSTANT_Long => ConstantType::Long (
				LongInfo::new(rdr.read_i64::<BigEndian>()?),
			),
			ConstantType::CONSTANT_Double => ConstantType::Double (
				DoubleInfo::new(rdr.read_f64::<BigEndian>()?),
			),
			ConstantType::CONSTANT_NameAndType => ConstantType::NameAndType (
				NameAndTypeInfo {
					name_index: rdr.read_u16::<BigEndian>()?,
					descriptor_index: rdr.read_u16::<BigEndian>()?
				},
			),
			ConstantType::CONSTANT_Utf8 => {
				let length = rdr.read_u16::<BigEndian>()? as usize;
				let bytes = rdr.read_nbytes(length)?;
				let utf = match mutf8::mutf8_to_utf8(bytes.as_slice()) {
					Cow::Borrowed(_data) => bytes.into(),
					Cow::Owned(data) => data.into_boxed_slice(),
				};
				
				let str = String::from_utf8_lossy(&utf);
				let str = String::from(&*str);
				ConstantType::Utf8 ( Utf8Info { str } )
			},
			ConstantType::CONSTANT_MethodHandle => {
				let kind = match rdr.read_u8()? {
					MethodHandleKind::REF_getField => MethodHandleKind::GetField,
					MethodHandleKind::REF_getStatic => MethodHandleKind::GetStatic,
					MethodHandleKind::REF_putField => MethodHandleKind::PutField,
					MethodHandleKind::REF_putStatic => MethodHandleKind::PutStatic,
					MethodHandleKind::REF_invokeVirtual => MethodHandleKind::InvokeVirtual,
					MethodHandleKind::REF_invokeStatic => MethodHandleKind::InvokeStatic,
					MethodHandleKind::REF_invokeSpecial => MethodHandleKind::InvokeSpecial,
					MethodHandleKind::REF_newInvokeSpecial => MethodHandleKind::NewInvokeSpecial,
					MethodHandleKind::REF_invokeInterface => MethodHandleKind::InvokeInterface,
					x => return Err(ParserError::other(format!("Unknown method handle type {}", x)))
				};
				let reference = rdr.read_u16::<BigEndian>()?;
				ConstantType::MethodHandle(MethodHandleInfo::new(kind, reference))
			},
			ConstantType::CONSTANT_MethodType => ConstantType::MethodType (
				MethodTypeInfo {
					descriptor_index: rdr.read_u16::<BigEndian>()?
				},
			),
			ConstantType::CONSTANT_Dynamic => ConstantType::Dynamic (
				DynamicInfo {
					bootstrap_method_attr_index: rdr.read_u16::<BigEndian>()?,
					name_and_type_index: rdr.read_u16::<BigEndian>()?
				},
			),
			ConstantType::CONSTANT_InvokeDynamic => ConstantType::InvokeDynamic (
				InvokeDynamicInfo {
					bootstrap_method_attr_index: rdr.read_u16::<BigEndian>()?,
					name_and_type_index: rdr.read_u16::<BigEndian>()?
				},
			),
			ConstantType::CONSTANT_Module => ConstantType::Module (
				ModuleInfo {
					name_index: rdr.read_u16::<BigEndian>()?
				},
			),
			ConstantType::CONSTANT_Package => ConstantType::Package (
				PackageInfo {
					name_index: rdr.read_u16::<BigEndian>()?
				},
			),
			_ => return Err(ParserError::unrecognised("constant tag", tag.to_string()))
		})
	}
	
	pub fn write<W: Write>(&self, wtr: &mut W) -> Result<()> {
		match self {
			ConstantType::Class(x) => {
				wtr.write_u8(ConstantType::CONSTANT_Class)?;
				wtr.write_u16::<BigEndian>(x.name_index)?;
			},
			ConstantType::Fieldref(x) => {
				wtr.write_u8(ConstantType::CONSTANT_Fieldref)?;
				wtr.write_u16::<BigEndian>(x.class_index)?;
				wtr.write_u16::<BigEndian>(x.name_and_type_index)?;
			}
			ConstantType::Methodref(x) => {
				wtr.write_u8(ConstantType::CONSTANT_Methodref)?;
				wtr.write_u16::<BigEndian>(x.class_index)?;
				wtr.write_u16::<BigEndian>(x.name_and_type_index)?;
			}
			ConstantType::InterfaceMethodref(x) => {
				wtr.write_u8(ConstantType::CONSTANT_InterfaceMethodref)?;
				wtr.write_u16::<BigEndian>(x.class_index)?;
				wtr.write_u16::<BigEndian>(x.name_and_type_index)?;
			}
			ConstantType::String(x) => {
				wtr.write_u8(ConstantType::CONSTANT_String)?;
				wtr.write_u16::<BigEndian>(x.utf_index)?;
			}
			ConstantType::Integer(x) => {
				wtr.write_u8(ConstantType::CONSTANT_Integer)?;
				wtr.write_i32::<BigEndian>(x.inner())?;
			}
			ConstantType::Float(x) => {
				wtr.write_u8(ConstantType::CONSTANT_Float)?;
				wtr.write_f32::<BigEndian>(x.inner())?;
			}
			ConstantType::Long(x) => {
				wtr.write_u8(ConstantType::CONSTANT_Long)?;
				wtr.write_i64::<BigEndian>(x.inner())?;
			}
			ConstantType::Double(x) => {
				wtr.write_u8(ConstantType::CONSTANT_Double)?;
				wtr.write_f64::<BigEndian>(x.inner())?;
			}
			ConstantType::NameAndType(x) => {
				wtr.write_u8(ConstantType::CONSTANT_NameAndType)?;
				wtr.write_u16::<BigEndian>(x.name_index)?;
				wtr.write_u16::<BigEndian>(x.descriptor_index)?;
			}
			ConstantType::Utf8(x) => {
				wtr.write_u8(ConstantType::CONSTANT_Utf8)?;
				let bytes = x.str.as_bytes();
				let mutf = match mutf8::utf8_to_mutf8(bytes) {
					Cow::Borrowed(_data) => bytes.into(),
					Cow::Owned(data) => data.into_boxed_slice(),
				};
				wtr.write_u16::<BigEndian>(mutf.len() as u16)?;
				wtr.write_all(&*mutf)?;
			}
			ConstantType::MethodHandle(x) => {
				wtr.write_u8(ConstantType::CONSTANT_MethodHandle)?;
				
				let reference_kind = match x.kind {
					MethodHandleKind::GetField => MethodHandleKind::REF_getField,
					MethodHandleKind::GetStatic => MethodHandleKind::REF_getStatic,
					MethodHandleKind::PutField => MethodHandleKind::REF_putField,
					MethodHandleKind::PutStatic => MethodHandleKind::REF_putStatic,
					MethodHandleKind::InvokeVirtual => MethodHandleKind::REF_invokeVirtual,
					MethodHandleKind::InvokeStatic => MethodHandleKind::REF_invokeStatic,
					MethodHandleKind::InvokeSpecial => MethodHandleKind::REF_invokeSpecial,
					MethodHandleKind::NewInvokeSpecial => MethodHandleKind::REF_newInvokeSpecial,
					MethodHandleKind::InvokeInterface => MethodHandleKind::REF_invokeInterface,
				};
				
				wtr.write_u8(reference_kind)?;
				wtr.write_u16::<BigEndian>(x.reference)?;
			}
			ConstantType::MethodType(x) => {
				wtr.write_u8(ConstantType::CONSTANT_MethodType)?;
				wtr.write_u16::<BigEndian>(x.descriptor_index)?;
			},
			ConstantType::Dynamic(x) => {
				wtr.write_u8(ConstantType::CONSTANT_Dynamic)?;
				wtr.write_u16::<BigEndian>(x.bootstrap_method_attr_index)?;
				wtr.write_u16::<BigEndian>(x.name_and_type_index)?;
			},
			ConstantType::InvokeDynamic(x) => {
				wtr.write_u8(ConstantType::CONSTANT_InvokeDynamic)?;
				wtr.write_u16::<BigEndian>(x.bootstrap_method_attr_index)?;
				wtr.write_u16::<BigEndian>(x.name_and_type_index)?;
			},
			ConstantType::Module(x) => {
				wtr.write_u8(ConstantType::CONSTANT_Module)?;
				wtr.write_u16::<BigEndian>(x.name_index)?;
			},
			ConstantType::Package(x) => {
				wtr.write_u8(ConstantType::CONSTANT_Package)?;
				wtr.write_u16::<BigEndian>(x.name_index)?;
			},
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
	index: CPIndex
}

impl ConstantPoolWriter {
	pub fn new() -> ConstantPoolWriter {
		ConstantPoolWriter {
			inner: LinkedHashMap::with_capacity(5),
			index: 1
		}
	}
	
	pub fn put(&mut self, constant: ConstantType) -> CPIndex {
		match self.inner.get(&constant) {
			Some(x) => *x,
			None => {
				let this_index = self.index;
				self.index += if constant.double_size() { 2	} else { 1 };
				self.inner.insert(constant, this_index);
				this_index
			}
		}
	}
	
	pub fn len(&self) -> u16 {
		self.index
	}
	
	pub fn class(&mut self, name_index: CPIndex) -> CPIndex {
		self.put(ConstantType::Class(ClassInfo::new(name_index)))
	}
	
	pub fn class_utf8<T: Into<String>>(&mut self, str: T) -> CPIndex {
		let utf = self.utf8(str);
		self.class(utf)
	}
	
	pub fn fieldref(&mut self, class_index: CPIndex, name_and_type_index: CPIndex) -> CPIndex {
		self.put(ConstantType::Fieldref(FieldRefInfo::new(class_index, name_and_type_index)))
	}
	
	pub fn methodref(&mut self, class_index: CPIndex, name_and_type_index: CPIndex) -> CPIndex {
		self.put(ConstantType::Methodref(MethodRefInfo::new(class_index, name_and_type_index)))
	}
	
	pub fn interfacemethodref(&mut self, class_index: CPIndex, name_and_type_index: CPIndex) -> CPIndex {
		self.put(ConstantType::InterfaceMethodref(MethodRefInfo::new(class_index, name_and_type_index)))
	}
	
	pub fn string(&mut self, string_index: CPIndex) -> CPIndex {
		self.put(ConstantType::String(StringInfo::new(string_index)))
	}
	
	pub fn string_utf<T: Into<String>>(&mut self, str: T) -> CPIndex {
		let utf = self.utf8(str);
		self.string(utf)
	}
	
	pub fn integer(&mut self, bytes: i32) -> CPIndex {
		self.put(ConstantType::Integer(IntegerInfo::new(bytes)))
	}
	
	pub fn float(&mut self, bytes: f32) -> CPIndex {
		self.put(ConstantType::Float(FloatInfo::new(bytes)))
	}
	
	pub fn long(&mut self, bytes: i64) -> CPIndex {
		self.put(ConstantType::Long(LongInfo::new(bytes)))
	}
	
	pub fn double(&mut self, bytes: f64) -> CPIndex {
		self.put(ConstantType::Double(DoubleInfo::new(bytes)))
	}
	
	pub fn nameandtype(&mut self, name_index: CPIndex, descriptor_index: CPIndex) -> CPIndex {
		self.put(ConstantType::NameAndType(NameAndTypeInfo::new(name_index, descriptor_index)))
	}
	
	pub fn utf8<T: Into<String>>(&mut self, str: T) -> CPIndex {
		self.put(ConstantType::Utf8(Utf8Info::new(str.into())))
	}
	
	pub fn methodhandle(&mut self, kind: MethodHandleKind, reference: CPIndex) -> CPIndex {
		self.put(ConstantType::MethodHandle(MethodHandleInfo::new(kind, reference)))
	}
	
	pub fn methodtype(&mut self, descriptor_index: CPIndex) -> CPIndex {
		self.put(ConstantType::MethodType(MethodTypeInfo::new(descriptor_index)))
	}
	
	pub fn methodtype_utf8<T: Into<String>>(&mut self, str: T) -> CPIndex {
		let utf = self.utf8(str);
		self.methodtype(utf)
	}
	
	pub fn dynamicinfo(&mut self, bootstrap_method_attr_index: CPIndex, name_and_type_index: CPIndex) -> CPIndex {
		self.put(ConstantType::Dynamic(DynamicInfo::new(bootstrap_method_attr_index, name_and_type_index)))
	}
	
	pub fn invokedynamicinfo(&mut self, bootstrap_method_attr_index: CPIndex, name_and_type_index: CPIndex) -> CPIndex {
		self.put(ConstantType::InvokeDynamic(InvokeDynamicInfo::new(bootstrap_method_attr_index, name_and_type_index)))
	}
	
	pub fn module(&mut self, name_index: CPIndex) -> CPIndex {
		self.put(ConstantType::Module(ModuleInfo::new(name_index)))
	}
	
	pub fn package(&mut self, name_index: CPIndex) -> CPIndex {
		self.put(ConstantType::Package(PackageInfo::new(name_index)))
	}
	
	pub fn write<W: Write>(&mut self, wtr: &mut W) -> Result<()> {
		wtr.write_u16::<BigEndian>(self.index as u16)?;
		for (constant, _index) in self.inner.iter() {
			constant.write(wtr)?;
		}
		
		Ok(())
	}
}
