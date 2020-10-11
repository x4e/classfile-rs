use crate::version::ClassVersion;
use crate::Serializable;
use std::io::{Write, Seek, Read};
use byteorder::{ReadBytesExt, BigEndian, WriteBytesExt};
use crate::constantpool::ConstantPool;
use crate::access::ClassAccessFlags;

#[derive(Clone, Debug, PartialEq)]
pub struct ClassFile {
	magic: u32, /// 0xCAFEBABE
	version: ClassVersion,
	constant_pool: ConstantPool,
	access: ClassAccessFlags,
}

impl Serializable for ClassFile {
	fn parse<R: Seek + Read>(rdr: &mut R) -> Self {
		let magic = rdr.read_u32::<BigEndian>().unwrap();
		assert_eq!(magic, 0xCAFEBABE, "Invalid class file magic");
		let version = ClassVersion::parse(rdr);
		let constant_pool = ConstantPool::parse(rdr);
		let access = ClassAccessFlags::parse(rdr);
		
		ClassFile {
			magic,
			version,
			constant_pool,
			access
		}
	}
	
	fn write<W: Seek + Write>(&self, wtr: &mut W) {
		wtr.write_u32::<BigEndian>(self.magic).unwrap();
		self.version.write(wtr);
	}
}
