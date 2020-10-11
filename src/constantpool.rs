use crate::Serializable;
use std::io::{Read, Write, Seek};
use byteorder::{ReadBytesExt, BigEndian, WriteBytesExt};
use std::borrow::Borrow;

pub type CPIndex = u16;

#[derive(Clone, Debug, PartialEq)]
pub struct ConstantPool {
	inner: Vec<Option<ConstantType>>
}

impl ConstantPool {
	pub fn get(&self, index: CPIndex) -> ConstantType {
		self.inner.get(index as usize).unwrap().unwrap()
	}
	
	pub fn class(&self, index: CPIndex) -> Option<ClassInfo> {
		match self.get(index) {
			ConstantType::Class(t) => Some(t),
			_ => None,
		}
	}
	
	pub fn fieldref(&self, index: CPIndex) -> Option<FieldInfo> {
		match self.get(index) {
			ConstantType::Fieldref(t) => Some(t),
			_ => None,
		}
	}
}

impl Serializable for ConstantPool {
	fn parse<R: Seek + Read>(rdr: &mut R) -> Self {
		let size = rdr.read_u16::<BigEndian>().unwrap() as usize;
		let mut inner: Vec<Option<ConstantType>> = vec![None; size];
		println!("CP: {}", size);
		for i in 1..size {
			let constant = ConstantType::parse(rdr);
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

#[derive(Clone, Debug, PartialEq)]
pub struct ClassInfo {
	name_index: CPIndex
}
#[derive(Clone, Debug, PartialEq)]
pub struct FieldInfo {
	class_index: CPIndex,
	name_and_type_index: CPIndex
}
#[derive(Clone, Debug, PartialEq)]
pub struct MethodRefInfo {
	class_index: CPIndex,
	name_and_type_index: CPIndex
}
#[derive(Clone, Debug, PartialEq)]
pub struct InterfaceMethodRefInfo {
	class_index: CPIndex,
	name_and_type_index: CPIndex
}
#[derive(Clone, Debug, PartialEq)]
pub struct StringInfo {
	string_index: CPIndex
}
#[derive(Clone, Debug, PartialEq)]
pub struct IntegerInfo {
	bytes: i32
}
#[derive(Clone, Debug, PartialEq)]
pub struct FloatInfo {
	bytes: f32
}
#[derive(Clone, Debug, PartialEq)]
pub struct LongInfo {
	bytes: i64
}
#[derive(Clone, Debug, PartialEq)]
pub struct DoubleInfo {
	bytes: f64
}
#[derive(Clone, Debug, PartialEq)]
pub struct NameAndTypeInfo {
	name_index: CPIndex,
	descriptor_index: CPIndex
}
#[derive(Clone, Debug, PartialEq)]
pub struct Utf8Info {
	str: String
}
#[derive(Clone, Debug, PartialEq)]
pub struct MethodHandleInfo {
	reference_kind: u8,
	reference_index: CPIndex
}
#[derive(Clone, Debug, PartialEq)]
pub struct MethodTypeInfo {
	descriptor_index: CPIndex
}
#[derive(Clone, Debug, PartialEq)]
pub struct DynamicInfo {
	bootstrap_method_attr_index: CPIndex,
	name_and_type_index: CPIndex
}
#[derive(Clone, Debug, PartialEq)]
pub struct InvokeDynamicInfo {
	bootstrap_method_attr_index: CPIndex,
	name_and_type_index: CPIndex
}
#[derive(Clone, Debug, PartialEq)]
pub struct ModuleInfo {
	name_index: CPIndex
}
#[derive(Clone, Debug, PartialEq)]
pub struct PackageInfo {
	name_index: CPIndex
}

#[derive(Clone, Debug, PartialEq)]
pub enum ConstantType {
	Class (ClassInfo),
	Fieldref (FieldInfo),
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

impl Serializable for ConstantType {
	fn parse<R: Seek + Read>(rdr: &mut R) -> Self {
		let tag = rdr.read_u8().unwrap();
		match tag {
			7 => ConstantType::Class {
				0: ClassInfo {
					name_index: rdr.read_u16::<BigEndian>().unwrap()
				},
			},
			9 => ConstantType::Fieldref {
				0: FieldInfo {
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
			15 => ConstantType::MethodHandle {
				0: MethodHandleInfo {
					reference_kind: rdr.read_u8().unwrap(),
					reference_index: rdr.read_u16::<BigEndian>().unwrap()
				},
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
	
	fn write<W: Seek + Write>(&self, wtr: &mut W) {
		match self {
			ConstantType::Class { .. } => {
				wtr.write_u8(7).unwrap()
			},
			_ => panic!("Not possible")
		}
	}
}
