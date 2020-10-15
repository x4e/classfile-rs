extern crate derive_more;
use std::io::{Seek, Read, Write};

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


pub trait Serializable {
	fn parse<R: Seek + Read>(rdr: &mut R) -> Self;
	fn write<W: Seek + Write>(&self, wtr: &mut W);
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::fs::File;
	use std::io::{BufReader};
	use std::time::Instant;
	use crate::classfile::ClassFile;
	
	fn read() -> ClassFile {
		// Read
		let start = Instant::now();
		let dir = "Class.class";
		let f = File::open(dir).unwrap();
		let mut reader = BufReader::new(f);
		ClassFile::parse(&mut reader)
	}
	
	#[test]
    fn print_read() {
		println!("{:#x?}", read());
    }
}
