use crate::Serializable;
use std::io::{Read, Seek, Write};
use std::cmp::{PartialOrd, Ordering};
use byteorder::{ReadBytesExt, BigEndian, WriteBytesExt};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord)]
pub struct ClassVersion {
	pub major: MajorVersion,
	pub minor: u16
}

impl PartialOrd for ClassVersion {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		let major = self.major.cmp(&other.major);
		if major == Ordering::Equal {
			return Some(self.minor.cmp(&other.minor));
		}
		Some(major)
	}
}

impl Serializable for ClassVersion {
	fn parse<R: Seek + Read>(rdr: &mut R) -> Self {
		let minor = rdr.read_u16::<BigEndian>().unwrap();
		let major = rdr.read_u16::<BigEndian>().unwrap();
		ClassVersion::new(major.into(), minor)
	}
	
	fn write<W: Seek + Write>(&self, wtr: &mut W) {
		wtr.write_u16::<BigEndian>(self.minor).unwrap();
		wtr.write_u16::<BigEndian>(self.major.into()).unwrap();
	}
}

#[allow(dead_code)]
impl ClassVersion {
	fn new_major(major: MajorVersion) -> Self {
		ClassVersion::new(major, 0)
	}
	fn new(major: MajorVersion, minor: u16) -> Self {
		ClassVersion {
			major, minor
		}
	}
}

#[allow(non_camel_case_types)]
#[repr(u16)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum MajorVersion {
	JDK_1_1 = 45,
	JDK_1_2 = 46,
	JDK_1_3 = 47,
	JDK_1_4 = 48,
	JAVA_5 = 49,
	JAVA_6 = 50,
	JAVA_7 = 51,
	JAVA_8 = 52,
	JAVA_9 = 53,
	JAVA_10 = 54,
	JAVA_11 = 55,
	JAVA_12 = 56,
	JAVA_13 = 57,
	JAVA_14 = 58,
	JAVA_15 = 59
}

impl Into<u16> for MajorVersion {
	fn into(self) -> u16 {
		self as u16
	}
}

impl Into<MajorVersion> for u16  {
	fn into(self) -> MajorVersion {
		match self {
			45 => MajorVersion::JDK_1_1,
			46 => MajorVersion::JDK_1_2,
			47 => MajorVersion::JDK_1_3,
			48 => MajorVersion::JDK_1_4,
			49 => MajorVersion::JAVA_5,
			50 => MajorVersion::JAVA_6,
			51 => MajorVersion::JAVA_7,
			52 => MajorVersion::JAVA_8,
			53 => MajorVersion::JAVA_9,
			54 => MajorVersion::JAVA_10,
			55 => MajorVersion::JAVA_11,
			56 => MajorVersion::JAVA_12,
			57 => MajorVersion::JAVA_13,
			58 => MajorVersion::JAVA_14,
			59 => MajorVersion::JAVA_15,
			_ => panic!("Unknown major classfile version {}", self)
		}
	}
}
