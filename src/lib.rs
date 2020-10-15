#![feature(try_trait)]
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


pub trait Serializable : Sized {
	fn parse<R: Seek + Read>(rdr: &mut R) -> Result<Self>;
	fn write<W: Seek + Write>(&self, wtr: &mut W) -> Result<()>;
}

#[cfg(test)]
mod tests {
	use std::fs::File;
	use std::io::{BufReader, BufWriter};
	use std::time::Instant;
	use crate::classfile::ClassFile;
	
	#[test]
    fn it_works() {
		// Read
		let start = Instant::now();
		let dir = "Class.class";
		let class = {
			let f = File::open(dir).unwrap();
			let mut reader = BufReader::new(f);
			ClassFile::parse(&mut reader)
		};
		
		let elapsed = start.elapsed();
		println!("{:#x?}", class);
		println!("Finished parsing {} in {:#?}", dir, elapsed);
		
		// Write
		{
			let dir = "TestOut.class";
			let f = File::create(dir).unwrap();
			let mut writer = BufWriter::new(f);
			class.write(&mut writer);
		}
    }
}
