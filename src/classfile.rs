use std::io::{Write, Seek, Read};
use byteorder::{ReadBytesExt, BigEndian, WriteBytesExt};
use crate::Serializable;
use crate::version::ClassVersion;
use crate::constantpool::{ConstantPool, CPIndex};
use crate::access::ClassAccessFlags;
use crate::field::Field;

#[derive(Clone, Debug, PartialEq)]
pub struct ClassFile {
	magic: u32, /// 0xCAFEBABE
	version: ClassVersion,
	constant_pool: ConstantPool,
	access_flags: ClassAccessFlags,
	this_class: String,
	super_class: String,
	interfaces: Vec<CPIndex>,
	fields: Vec<Field>
}

impl Serializable for ClassFile {
	fn parse<R: Seek + Read>(rdr: &mut R) -> Self {
		let magic = rdr.read_u32::<BigEndian>().unwrap();
		assert_eq!(magic, 0xCAFEBABE, "Invalid class file magic");
		let version = ClassVersion::parse(rdr);
		let constant_pool = ConstantPool::parse(rdr);
		let access_flags = ClassAccessFlags::parse(rdr);
		let this_class = constant_pool.utf8(constant_pool.class(rdr.read_u16::<BigEndian>().unwrap()).unwrap().name_index).unwrap().str.clone();
		let super_class = constant_pool.utf8(constant_pool.class(rdr.read_u16::<BigEndian>().unwrap()).unwrap().name_index).unwrap().str.clone();
		
		let num_interfaces = rdr.read_u16::<BigEndian>().unwrap() as usize;
		let mut interfaces: Vec<CPIndex> = Vec::with_capacity(num_interfaces);
		for _ in 0..num_interfaces {
			interfaces.push(rdr.read_u16::<BigEndian>().unwrap());
		}
		
		let num_fields = rdr.read_u16::<BigEndian>().unwrap() as usize;
		let mut fields: Vec<Field> = Vec::with_capacity(num_fields);
		for _ in 0..num_fields {
			fields.push(Field::parse(rdr, &constant_pool));
		}
		
		ClassFile {
			magic,
			version,
			constant_pool,
			access_flags,
			this_class,
			super_class,
			interfaces,
			fields
		}
	}
	
	fn write<W: Seek + Write>(&self, wtr: &mut W) {
		wtr.write_u32::<BigEndian>(self.magic).unwrap();
		self.version.write(wtr);
	}
}
