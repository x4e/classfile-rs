use crate::Serializable;
use std::io::{Read, Write, Seek};
use byteorder::{ReadBytesExt, BigEndian, WriteBytesExt};
use std::borrow::Borrow;
use derive_more::Constructor;

pub type CPIndex = u16;

#[derive(Clone, Debug, PartialEq)]
pub struct ConstantPool {
	inner: Vec<Option<ConstantType>>
}

#[allow(dead_code)]
impl ConstantPool {
	pub fn new() -> ConstantPool {
		ConstantPool {
			inner: Vec::with_capacity(12)
		}
	}
	
	pub fn get(&self, index: CPIndex) -> Option<&ConstantType> {
		match self.inner.get(index as usize) {
			Some(Some(x)) => Some(x),
			_ => None
		}
	}
	
	pub fn class(&self, index: CPIndex) -> Result<&ClassInfo, String> {
		match self.get(index) {
			Some(ConstantType::Class(t)) => Ok(t),
			x => Err(format!("Index {} is not a Class, found {:?}", index, x)),
		}
	}
	
	pub fn fieldref(&self, index: CPIndex) -> Result<&FieldRefInfo, String> {
		match self.get(index) {
			Some(ConstantType::Fieldref(t)) => Ok(t),
			x => Err(format!("Index {} is not a Fieldref, found {:?}", index, x)),
		}
	}
	
	pub fn methodref(&self, index: CPIndex) -> Result<&MethodRefInfo, String> {
		match self.get(index) {
			Some(ConstantType::Methodref(t)) => Ok(t),
			x => Err(format!("Index {} is not a Methodref, found {:?}", index, x)),
		}
	}
	
	pub fn interfacemethodref(&self, index: CPIndex) -> Result<&InterfaceMethodRefInfo, String> {
		match self.get(index) {
			Some(ConstantType::InterfaceMethodref(t)) => Ok(t),
			x => Err(format!("Index {} is not an InterfaceMethodref, found {:?}", index, x)),
		}
	}
	
	pub fn string(&self, index: CPIndex) -> Result<&StringInfo, String> {
		match self.get(index) {
			Some(ConstantType::String(t)) => Ok(t),
			x => Err(format!("Index {} is not a String, found {:?}", index, x)),
		}
	}
	
	pub fn integer(&self, index: CPIndex) -> Result<&IntegerInfo, String> {
		match self.get(index) {
			Some(ConstantType::Integer(t)) => Ok(t),
			x => Err(format!("Index {} is not an Integer, found {:?}", index, x)),
		}
	}
	
	pub fn float(&self, index: CPIndex) -> Result<&FloatInfo, String> {
		match self.get(index) {
			Some(ConstantType::Float(t)) => Ok(t),
			x => Err(format!("Index {} is not a String, found {:?}", index, x)),
		}
	}
	
	pub fn long(&self, index: CPIndex) -> Result<&LongInfo, String> {
		match self.get(index) {
			Some(ConstantType::Long(t)) => Ok(t),
			x => Err(format!("Index {} is not a Long, found {:?}", index, x)),
		}
	}
	
	pub fn double(&self, index: CPIndex) -> Result<&DoubleInfo, String> {
		match self.get(index) {
			Some(ConstantType::Double(t)) => Ok(t),
			x => Err(format!("Index {} is not a Double, found {:?}", index, x)),
		}
	}
	
	pub fn nameandtype(&self, index: CPIndex) -> Result<&NameAndTypeInfo, String> {
		match self.get(index) {
			Some(ConstantType::NameAndType(t)) => Ok(t),
			x => Err(format!("Index {} is not a NameAndType, found {:?}", index, x)),
		}
	}
	
	pub fn utf8(&self, index: CPIndex) -> Result<&Utf8Info, String> {
		match self.get(index) {
			Some(ConstantType::Utf8(t)) => Ok(t),
			x => Err(format!("Index {} is not a Utf8, found {:?}", index, x)),
		}
	}
	
	pub fn methodhandle(&self, index: CPIndex) -> Result<&MethodHandleInfo, String> {
		match self.get(index) {
			Some(ConstantType::MethodHandle(t)) => Ok(t),
			x => Err(format!("Index {} is not a MethodHandle, found {:?}", index, x)),
		}
	}
	
	pub fn methodtype(&self, index: CPIndex) -> Result<&MethodTypeInfo, String> {
		match self.get(index) {
			Some(ConstantType::MethodType(t)) => Ok(t),
			x => Err(format!("Index {} is not a MethodHandle, found {:?}", index, x)),
		}
	}
	
	pub fn dynamicinfo(&self, index: CPIndex) -> Result<&DynamicInfo, String> {
		match self.get(index) {
			Some(ConstantType::Dynamic(t)) => Ok(t),
			x => Err(format!("Index {} is not a Dynamic, found {:?}", index, x)),
		}
	}
	
	pub fn invokedynamicinfo(&self, index: CPIndex) -> Result<&InvokeDynamicInfo, String> {
		match self.get(index) {
			Some(ConstantType::InvokeDynamic(t)) => Ok(t),
			x => Err(format!("Index {} is not a Dynamic, found {:?}", index, x)),
		}
	}
	
	pub fn module(&self, index: CPIndex) -> Result<&ModuleInfo, String> {
		match self.get(index) {
			Some(ConstantType::Module(t)) => Ok(t),
			x => Err(format!("Index {} is not a Module, found {:?}", index, x)),
		}
	}
	
	pub fn package(&self, index: CPIndex) -> Result<&PackageInfo, String> {
		match self.get(index) {
			Some(ConstantType::Package(t)) => Ok(t),
			x => Err(format!("Index {} is not a Package, found {:?}", index, x)),
		}
	}
}

impl Serializable for ConstantPool {
	fn parse<R: Seek + Read>(rdr: &mut R) -> Self {
		let size = rdr.read_u16::<BigEndian>().unwrap() as usize;
		let mut inner: Vec<Option<ConstantType>> = vec![None; size];
		let mut skip = false;
		for i in 1..size {
			if skip {
				skip = false;
				continue
			}
			let constant = ConstantType::parse(rdr, &inner);
			match constant {
				ConstantType::Double(..) | ConstantType::Long(..) => {
					skip = true;
				}
				_ => {}
			}
			inner[i] = Some(constant);
		}
		
		ConstantPool {
			inner
		}
	}
	
	fn write<W: Seek + Write>(&self, wtr: &mut W) {
		wtr.write_u16::<BigEndian>(self.inner.len() as u16).unwrap();
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

#[derive(Clone, Debug, PartialEq)]
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
	pub fn parse<R: Seek + Read>(rdr: &mut R, constants: &Vec<Option<ConstantType>>) -> Self {
		let tag = rdr.read_u8().unwrap();
		match tag {
			7 => ConstantType::Class {
				0: ClassInfo {
					name_index: rdr.read_u16::<BigEndian>().unwrap()
				},
			},
			9 => ConstantType::Fieldref {
				0: FieldRefInfo {
					class_index: rdr.read_u16::<BigEndian>().unwrap(),
					name_and_type_index: rdr.read_u16::<BigEndian>().unwrap()
				},
			},
			10 => ConstantType::Methodref {
				0: MethodRefInfo {
					class_index: rdr.read_u16::<BigEndian>().unwrap(),
					name_and_type_index: rdr.read_u16::<BigEndian>().unwrap()
				},
			},
			11 => ConstantType::InterfaceMethodref {
				0: InterfaceMethodRefInfo {
					class_index: rdr.read_u16::<BigEndian>().unwrap(),
					name_and_type_index: rdr.read_u16::<BigEndian>().unwrap()
				},
			},
			8 => ConstantType::String {
				0: StringInfo {
					string_index: rdr.read_u16::<BigEndian>().unwrap()
				},
			},
			3 => ConstantType::Integer {
				0: IntegerInfo {
					bytes: rdr.read_i32::<BigEndian>().unwrap()
				},
			},
			4 => ConstantType::Float {
				0: FloatInfo {
					bytes: rdr.read_f32::<BigEndian>().unwrap()
				},
			},
			5 => ConstantType::Long {
				0: LongInfo {
					bytes: rdr.read_i64::<BigEndian>().unwrap()
				},
			},
			6 => ConstantType::Double {
				0: DoubleInfo {
					bytes: rdr.read_f64::<BigEndian>().unwrap()
				},
			},
			12 => ConstantType::NameAndType {
				0: NameAndTypeInfo {
					name_index: rdr.read_u16::<BigEndian>().unwrap(),
					descriptor_index: rdr.read_u16::<BigEndian>().unwrap()
				},
			},
			1 => {
				let length = rdr.read_u16::<BigEndian>().unwrap() as usize;
				let mut bytes: Vec<u8> = Vec::with_capacity(length);
				for _ in 0..length {
					bytes.push(rdr.read_u8().unwrap());
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
						let c2 = rdr.read_u8().unwrap();
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
				let reference_kind = rdr.read_u8().unwrap();
				let reference_index = rdr.read_u16::<BigEndian>().unwrap() as usize;
				let reference = constants.get(reference_index).unwrap().as_ref().unwrap();
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
					descriptor_index: rdr.read_u16::<BigEndian>().unwrap()
				},
			},
			17 => ConstantType::Dynamic {
				0: DynamicInfo {
					bootstrap_method_attr_index: rdr.read_u16::<BigEndian>().unwrap(),
					name_and_type_index: rdr.read_u16::<BigEndian>().unwrap()
				},
			},
			18 => ConstantType::InvokeDynamic {
				0: InvokeDynamicInfo {
					bootstrap_method_attr_index: rdr.read_u16::<BigEndian>().unwrap(),
					name_and_type_index: rdr.read_u16::<BigEndian>().unwrap()
				},
			},
			19 => ConstantType::Module {
				0: ModuleInfo {
					name_index: rdr.read_u16::<BigEndian>().unwrap()
				},
			},
			20 => ConstantType::Package {
				0: PackageInfo {
					name_index: rdr.read_u16::<BigEndian>().unwrap()
				},
			},
			_ => panic!("Unknown Constant tag {}", tag)
		}
	}
	
	pub fn write<W: Seek + Write>(&self, wtr: &mut W) {
		match self {
			ConstantType::Class { .. } => {
				wtr.write_u8(7).unwrap()
			},
			_ => panic!("Not possible")
		}
	}
}
