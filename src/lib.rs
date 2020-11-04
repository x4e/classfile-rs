extern crate derive_more;
use std::io::{Read, Write};
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
	fn parse<R: Read>(rdr: &mut R) -> Result<Self>;
	fn write<W: Write>(&self, wtr: &mut W) -> Result<()>;
}

#[cfg(test)]
mod tests {
	use std::fs::{self, File, DirEntry};
	use std::io::BufReader;
	use std::process::Command;
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
	
	fn walk(dir: &str, op: &dyn Fn(DirEntry) -> Result<()>) -> Result<()> {
		for entry in fs::read_dir(dir)? {
			let entry = entry?;
			op(entry)?;
		}
		Ok(())
	}
	
	#[test]
	fn test_classes() -> Result<()> {
		walk("classes/benchmarking/", &|entry| {
			let path = entry.path();
			if path.is_file() {
				let extension = path.extension().unwrap().to_str().unwrap();
				if extension == "class" {
					read(path.into_os_string().to_str().unwrap()).unwrap();
				}
			}
			Ok(())
		})?;/*
		walk("classes/testing/", &|entry| {
			let path = entry.path();
			if path.is_file() {
				let extension = path.extension().unwrap().to_str().unwrap();
				if extension == "class" {
					fs::remove_file(path)?;
				}
			}
			Ok(())
		})?;
		walk("classes/testing/", &|entry| {
			let path = entry.path();
			if path.is_file() {
				let extension = path.extension().unwrap().to_str().unwrap();
				if extension == "java" {
					let output = Command::new("javac")
						.args(&[path.into_os_string().to_str().unwrap()])
						.output()
						.unwrap();
					if !output.stderr.is_empty() {
						panic!("{}", String::from_utf8(output.stderr).unwrap());
					}
				}
			}
			Ok(())
		})?;
		walk("classes/testing/", &|entry| {
			let path = entry.path();
			if path.is_file() {
				let extension = path.extension().unwrap().to_str().unwrap();
				if extension == "class" {
					print_read(path.into_os_string().to_str().unwrap());
				}
			}
			Ok(())
		})?;*/
		Ok(())
	}
}
