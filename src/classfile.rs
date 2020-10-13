use std::io::{Write, Seek, Read};
use byteorder::{ReadBytesExt, BigEndian, WriteBytesExt};
use crate::Serializable;
use crate::version::ClassVersion;
use crate::constantpool::{ConstantPool};
use crate::access::ClassAccessFlags;
use crate::field::{Field, Fields};
use crate::method::{Methods, Method};

#[derive(Clone, Debug, PartialEq)]
pub struct ClassFile {
	magic: u32, /// 0xCAFEBABE
	version: ClassVersion,
	access_flags: ClassAccessFlags,
	this_class: String,
	super_class: String,
	interfaces: Vec<String>,
	fields: Vec<Field>,
	methods: Vec<Method>
}

impl ClassFile {
	pub fn parse<R: Seek + Read>(rdr: &mut R) -> Self {
		let magic = rdr.read_u32::<BigEndian>().unwrap();
		if magic != 0xCAFEBABE {
			panic!("Invalid class file magic {}", magic);
		}
		let version = ClassVersion::parse(rdr);
		let constant_pool = ConstantPool::parse(rdr);
		let access_flags = ClassAccessFlags::parse(rdr);
		let this_class = constant_pool.utf8(constant_pool.class(rdr.read_u16::<BigEndian>().unwrap()).unwrap().name_index).unwrap().str.clone();
		let super_class = constant_pool.utf8(constant_pool.class(rdr.read_u16::<BigEndian>().unwrap()).unwrap().name_index).unwrap().str.clone();
		
		let num_interfaces = rdr.read_u16::<BigEndian>().unwrap() as usize;
		let mut interfaces: Vec<String> = Vec::with_capacity(num_interfaces);
		for _ in 0..num_interfaces {
			interfaces.push(constant_pool.utf8(constant_pool.class(rdr.read_u16::<BigEndian>().unwrap()).unwrap().name_index).unwrap().str.clone());
		}
		
		let fields = Fields::parse(rdr, &version, &constant_pool);
		let methods = Methods::parse(rdr, &version, &constant_pool);
		
		ClassFile {
			magic,
			version,
			access_flags,
			this_class,
			super_class,
			interfaces,
			fields,
			methods
		}
	}
	
	pub fn write<W: Seek + Write>(&self, wtr: &mut W) {
		wtr.write_u32::<BigEndian>(self.magic).unwrap();
		self.version.write(wtr);
		self.access_flags.write(wtr);
		
		let constant_pool = ConstantPool::new();
		
		Fields::write(wtr, &self.fields, &constant_pool);
		Methods::write(wtr, &self.methods, &constant_pool);
	}
}
