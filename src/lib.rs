extern crate derive_more;
use std::io::{Seek, Read, Write};
use error::Result;

pub mod classfile;
pub mod constantpool;
pub mod version;
pub mod access;
pub mod attributes;
pub mod field;
pub mod method;
pub mod code;
pub mod ast;
pub mod insnlist;
pub mod error;
mod utils;


pub trait Serializable : Sized {
	fn parse<R: Seek + Read>(rdr: &mut R) -> Result<Self>;
	fn write<W: Seek + Write>(&self, wtr: &mut W) -> Result<()>;
}

#[cfg(test)]
mod tests {
	use std::fs::File;
	use std::io::BufReader;
	use crate::classfile::ClassFile;
	use crate::error::Result;
	
	fn read(dir: &str) -> Result<ClassFile> {
		// Read
		let f = File::open(dir).unwrap();
		let mut reader = BufReader::new(f);
		ClassFile::parse(&mut reader)
	}
	
    fn print_read(dir: &str) {
		println!("{:#x?}", read(dir));
    }
	
	#[test]
	fn class_class() {
		read("Class.class");
	}
	
	#[test]
	fn object_class() {
		print_read("Object.class");
	}
}
