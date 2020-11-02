use std::io::{Write, Seek, Read, Cursor};
use byteorder::{ReadBytesExt, BigEndian, WriteBytesExt};
use crate::Serializable;
use crate::version::ClassVersion;
use crate::constantpool::{ConstantPool, ConstantPoolWriter};
use crate::access::ClassAccessFlags;
use crate::field::{Field, Fields};
use crate::method::{Methods, Method};
use crate::error::{Result, ParserError};

#[derive(Clone, Debug, PartialEq)]
pub struct ClassFile {
	/// 0xCAFEBABE
	magic: u32,
	version: ClassVersion,
	access_flags: ClassAccessFlags,
	this_class: String,
	/// Can be None for example for java/lang/Object
	super_class: Option<String>,
	interfaces: Vec<String>,
	fields: Vec<Field>,
	methods: Vec<Method>
}

impl ClassFile {
	pub fn parse<R: Seek + Read>(rdr: &mut R) -> Result<Self> {
		let magic = rdr.read_u32::<BigEndian>()?;
		if magic != 0xCAFEBABE {
			return Err(ParserError::unrecognised("header", magic.to_string()));
		}
		let version = ClassVersion::parse(rdr)?;
		let constant_pool = ConstantPool::parse(rdr)?;
		let access_flags = ClassAccessFlags::parse(rdr)?;
		let this_class = constant_pool.utf8(constant_pool.class(rdr.read_u16::<BigEndian>()?)?.name_index)?.str.clone();
		let super_class = match rdr.read_u16::<BigEndian>()? {
			0 => None,
			i => Some(constant_pool.utf8(constant_pool.class(i)?.name_index)?.str.clone())
		};
		
		let num_interfaces = rdr.read_u16::<BigEndian>()? as usize;
		let mut interfaces: Vec<String> = Vec::with_capacity(num_interfaces);
		for _ in 0..num_interfaces {
			interfaces.push(constant_pool.utf8(constant_pool.class(rdr.read_u16::<BigEndian>()?)?.name_index)?.str.clone());
		}
		
		let fields = Fields::parse(rdr, &version, &constant_pool)?;
		let methods = Methods::parse(rdr, &version, &constant_pool)?;
		
		Ok(ClassFile {
			magic,
			version,
			access_flags,
			this_class,
			super_class,
			interfaces,
			fields,
			methods
		})
	}
	
	pub fn write<W: Seek + Write>(&self, wtr: &mut W) -> Result<()> {
		wtr.write_u32::<BigEndian>(self.magic)?;
		self.version.write(wtr)?;
		
		let mut constant_pool = ConstantPoolWriter::new();
		
		// we need to write fields/methods etc after the constant pool, however they rely upon
		// mutable access to the constant pool. therefore we will write them to memory and then to
		// the wtr parameter
		let mut buff: Vec<u8> = Vec::with_capacity(2 + (self.fields.len() * 8) + (self.methods.len() * 8));
		let mut cursor = Cursor::new(buff);
		self.access_flags.write(&mut cursor)?;
		
		// this class
		wtr.write_u16::<BigEndian>(constant_pool.class(constant_pool.utf8(self.this_class.clone())))?;
		// super class
		if let Some(x) = &self.super_class {
			wtr.write_u16::<BigEndian>(constant_pool.class(constant_pool.utf8(x.clone())))?;
		} else {
			wtr.write_u16::<BigEndian>(0)?;
		}
		
		Fields::write(wtr, &self.fields, &constant_pool)?;
		Methods::write(wtr, &self.methods, &constant_pool)?;
		Ok(())
	}
}
