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
	
	fn read() -> Result<ClassFile> {
		// Read
		let dir = "Object.class";
		let f = File::open(dir).unwrap();
		let mut reader = BufReader::new(f);
		ClassFile::parse(&mut reader)
	}
	
	#[test]
    fn print_read() {
		println!("{:#x?}", read());
    }
}
