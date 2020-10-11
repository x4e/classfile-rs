use crate::Serializable;
use std::io::{Read, Write, Seek};
use byteorder::{ReadBytesExt, BigEndian, WriteBytesExt};
use std::borrow::Borrow;
use crate::constantpool::ConstantType::{MethodHandle, MethodType, Dynamic, InvokeDynamic, Package, Module};

type CPIndex = u16;

#[derive(Clone, Debug, PartialEq)]
pub struct ConstantPool {
	inner: Vec<Option<ConstantType>>
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
pub enum ConstantType {
	Class {
		name_index: CPIndex
	},
	Fieldref {
		class_index: CPIndex,
		name_and_type_index: CPIndex
	},
	Methodref {
		class_index: CPIndex,
		name_and_type_index: CPIndex
	},
	InterfaceMethodref {
		class_index: CPIndex,
		name_and_type_index: CPIndex
	},
	String {
		string_index: CPIndex
	},
	Integer {
		bytes: i32
	},
	Float {
		bytes: f32
	},
	Long {
		bytes: i64
	},
	Double {
		bytes: f64
	},
	NameAndType {
		name_index: CPIndex,
		descriptor_index: CPIndex
	},
	Utf8 {
		str: String
	},
	MethodHandle {
		reference_kind: u8,
		reference_index: CPIndex
	},
	MethodType {
		descriptor_index: CPIndex
	},
	Dynamic {
		bootstrap_method_attr_index: CPIndex,
		name_and_type_index: CPIndex
	},
	InvokeDynamic {
		bootstrap_method_attr_index: CPIndex,
		name_and_type_index: CPIndex
	},
	Module {
		name_index: CPIndex
	},
	Package {
		name_index: CPIndex
	}
}

impl Serializable for ConstantType {
	fn parse<R: Seek + Read>(rdr: &mut R) -> Self {
		let tag = rdr.read_u8().unwrap();
		match tag {
			7 => ConstantType::Class {
				name_index: rdr.read_u16::<BigEndian>().unwrap()
			},
			9 => ConstantType::Fieldref {
				class_index: rdr.read_u16::<BigEndian>().unwrap(),
				name_and_type_index: rdr.read_u16::<BigEndian>().unwrap()
			},
			10 => ConstantType::Methodref {
				class_index: rdr.read_u16::<BigEndian>().unwrap(),
				name_and_type_index: rdr.read_u16::<BigEndian>().unwrap()
			},
			11 => ConstantType::InterfaceMethodref {
				class_index: rdr.read_u16::<BigEndian>().unwrap(),
				name_and_type_index: rdr.read_u16::<BigEndian>().unwrap()
			},
			8 => ConstantType::String {
				string_index: rdr.read_u16::<BigEndian>().unwrap()
			},
			3 => ConstantType::Integer {
				bytes: rdr.read_i32::<BigEndian>().unwrap()
			},
			4 => ConstantType::Float {
				bytes: rdr.read_f32::<BigEndian>().unwrap()
			},
			5 => ConstantType::Long {
				bytes: rdr.read_i64::<BigEndian>().unwrap()
			},
			6 => ConstantType::Double {
				bytes: rdr.read_f64::<BigEndian>().unwrap()
			},
			12 => ConstantType::NameAndType {
				name_index: rdr.read_u16::<BigEndian>().unwrap(),
				descriptor_index: rdr.read_u16::<BigEndian>().unwrap()
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
					str
				}
			},
			15 => MethodHandle {
				reference_kind: rdr.read_u8().unwrap(),
				reference_index: rdr.read_u16::<BigEndian>().unwrap()
			},
			16 => MethodType {
				descriptor_index: rdr.read_u16::<BigEndian>().unwrap()
			},
			17 => Dynamic {
				bootstrap_method_attr_index: rdr.read_u16::<BigEndian>().unwrap(),
				name_and_type_index: rdr.read_u16::<BigEndian>().unwrap()
			},
			18 => InvokeDynamic {
				bootstrap_method_attr_index: rdr.read_u16::<BigEndian>().unwrap(),
				name_and_type_index: rdr.read_u16::<BigEndian>().unwrap()
			},
			19 => Module {
				name_index: rdr.read_u16::<BigEndian>().unwrap()
			},
			20 => Package {
				name_index: rdr.read_u16::<BigEndian>().unwrap()
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
