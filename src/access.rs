#![allow(dead_code)]

use crate::Serializable;
use std::io::{Seek, Read, Write};
use byteorder::{ReadBytesExt, BigEndian, WriteBytesExt};
use std::fmt::{Debug, Formatter, Error};
use std::borrow::Borrow;

#[derive(Clone, Debug, PartialEq)]
pub struct ClassAccessFlags {
	flags: Vec<AccessFlag>
}

impl Serializable for ClassAccessFlags {
	fn parse<R: Seek + Read>(rdr: &mut R) -> Self {
		let flag = rdr.read_u16::<BigEndian>().unwrap();
		let mut flags: Vec<AccessFlag> = Vec::new();
		for access in CLASS_ACCESSES.iter() {
			if flag & access.0 != 0 {
				flags.push(access.clone());
			}
		}
		ClassAccessFlags {
			flags
		}
	}
	
	fn write<W: Seek + Write>(&self, wtr: &mut W) {
		let mut flag = 0u16;
		for access in self.flags.iter() {
			flag &= access.0;
		}
		wtr.write_u16::<BigEndian>(flag).unwrap();
	}
}

#[derive(Clone, Debug, PartialEq)]
pub struct FieldAccessFlags {
	flags: Vec<AccessFlag>
}

impl Serializable for FieldAccessFlags {
	fn parse<R: Seek + Read>(rdr: &mut R) -> Self {
		let flag = rdr.read_u16::<BigEndian>().unwrap();
		let mut flags: Vec<AccessFlag> = Vec::new();
		for access in FIELD_ACCESSES.iter() {
			if flag & access.0 != 0 {
				flags.push(access.clone());
			}
		}
		FieldAccessFlags {
			flags
		}
	}
	
	fn write<W: Seek + Write>(&self, wtr: &mut W) {
		let mut flag = 0u16;
		for access in self.flags.iter() {
			flag &= access.0;
		}
		wtr.write_u16::<BigEndian>(flag).unwrap();
	}
}

#[derive(Clone, Debug, PartialEq)]
pub struct MethodAccessFlags {
	flags: Vec<AccessFlag>
}

impl Serializable for MethodAccessFlags {
	fn parse<R: Seek + Read>(rdr: &mut R) -> Self {
		let flag = rdr.read_u16::<BigEndian>().unwrap();
		let mut flags: Vec<AccessFlag> = Vec::new();
		for access in METHOD_ACCESSES.iter() {
			if flag & access.0 != 0 {
				flags.push(access.clone());
			}
		}
		MethodAccessFlags {
			flags
		}
	}
	
	fn write<W: Seek + Write>(&self, wtr: &mut W) {
		let mut flag = 0u16;
		for access in self.flags.iter() {
			flag &= access.0;
		}
		wtr.write_u16::<BigEndian>(flag).unwrap();
	}
}

#[derive(Clone, Debug, PartialEq)]
pub struct InnerClassAccessFlags {
	flags: Vec<AccessFlag>
}

impl Serializable for InnerClassAccessFlags {
	fn parse<R: Seek + Read>(rdr: &mut R) -> Self {
		let flag = rdr.read_u16::<BigEndian>().unwrap();
		let mut flags: Vec<AccessFlag> = Vec::new();
		for access in INNERCLASS_ACCESSES.iter() {
			if flag & access.0 != 0 {
				flags.push(access.clone());
			}
		}
		InnerClassAccessFlags {
			flags
		}
	}
	
	fn write<W: Seek + Write>(&self, wtr: &mut W) {
		let mut flag = 0u16;
		for access in self.flags.iter() {
			flag &= access.0;
		}
		wtr.write_u16::<BigEndian>(flag).unwrap();
	}
}

pub static ACC_PUBLIC: AccessFlag = AccessFlag::new(0x0001, "public");
pub static ACC_PRIVATE: AccessFlag = AccessFlag::new(0x0002, "private");
pub static ACC_PROTECTED: AccessFlag = AccessFlag::new(0x0004, "protected");
pub static ACC_STATIC: AccessFlag = AccessFlag::new(0x0008, "static");
pub static ACC_FINAL: AccessFlag = AccessFlag::new(0x0010, "final");
pub static ACC_SYNCHRONIZED: AccessFlag = AccessFlag::new(0x0020, "synchronized");
pub static ACC_SUPER: AccessFlag = AccessFlag::new(0x0020, "super");
pub static ACC_BRIDGE: AccessFlag = AccessFlag::new(0x0040, "bridge");
pub static ACC_VOLATILE: AccessFlag = AccessFlag::new(0x0040, "volatile");
pub static ACC_VARARGS: AccessFlag = AccessFlag::new(0x0080, "varargs");
pub static ACC_TRANSIENT: AccessFlag = AccessFlag::new(0x0080, "transient");
pub static ACC_NATIVE: AccessFlag = AccessFlag::new(0x0100, "native");
pub static ACC_INTERFACE: AccessFlag = AccessFlag::new(0x0100, "native");
pub static ACC_ABSTRACT: AccessFlag = AccessFlag::new(0x0400, "abstract");
pub static ACC_STRICT: AccessFlag = AccessFlag::new(0x0800, "strictfp");
pub static ACC_SYNTHETIC: AccessFlag = AccessFlag::new(0x1000, "synthetic");
pub static ACC_ANNOTATION: AccessFlag = AccessFlag::new(0x2000, "annotation");
pub static ACC_ENUM: AccessFlag = AccessFlag::new(0x4000, "enum");
pub static ACC_MODULE: AccessFlag = AccessFlag::new(0x8000, "module");
pub static ACC_MANDATED: AccessFlag = AccessFlag::new(0x8000, "mandated");

static CLASS_ACCESSES: [AccessFlag; 9] =
	[ACC_PUBLIC, ACC_FINAL, ACC_SUPER, ACC_INTERFACE, ACC_ABSTRACT, ACC_SYNTHETIC, ACC_ANNOTATION,
		ACC_ENUM, ACC_MODULE];

static FIELD_ACCESSES: [AccessFlag; 9] =
	[ACC_PUBLIC, ACC_PRIVATE, ACC_PROTECTED, ACC_STATIC, ACC_FINAL, ACC_VOLATILE, ACC_TRANSIENT,
		ACC_SYNTHETIC, ACC_ENUM];

static METHOD_ACCESSES: [AccessFlag; 12] =
	[ACC_PUBLIC, ACC_PRIVATE, ACC_PROTECTED, ACC_STATIC, ACC_FINAL, ACC_SYNCHRONIZED, ACC_BRIDGE,
		ACC_VARARGS, ACC_NATIVE, ACC_ABSTRACT, ACC_STRICT, ACC_SYNTHETIC];

static INNERCLASS_ACCESSES: [AccessFlag; 10] =
	[ACC_PUBLIC, ACC_PRIVATE, ACC_PROTECTED, ACC_STATIC, ACC_FINAL, ACC_INTERFACE, ACC_ABSTRACT,
		ACC_SYNTHETIC, ACC_ANNOTATION, ACC_ENUM];

#[derive(Copy, Clone, Eq)]
pub struct AccessFlag (
	u16,
	&'static str
);

impl PartialEq for AccessFlag {
	fn eq(&self, other: &Self) -> bool {
		self.0.eq(&other.0)
	}
}

impl AccessFlag {
	const fn new(flag: u16, name: &'static str) -> Self {
		AccessFlag (flag, name)
	}
}

impl ToString for AccessFlag {
	fn to_string(&self) -> String {
		String::from(self.1)
	}
}

impl Debug for AccessFlag {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
		f.write_str(self.to_string().borrow()).unwrap();
		Ok(())
	}
}
