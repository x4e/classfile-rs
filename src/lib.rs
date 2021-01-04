extern crate derive_more;
extern crate bitflags;
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
pub mod types;
mod utils;


pub trait Serializable : Sized {
	fn parse<R: Read>(rdr: &mut R) -> Result<Self>;
	fn write<W: Write>(&self, wtr: &mut W) -> Result<()>;
}

#[cfg(test)]
mod tests {
	use crate::classfile::ClassFile;
	use crate::error::Result;
	use std::fs::{self, File, DirEntry, OpenOptions};
	use std::io::{BufReader, BufWriter};
	use std::process::Command;
	
	fn read(dir: &str) -> Result<ClassFile> {
		// Read
		let f = File::open(dir).unwrap();
		let mut reader = BufReader::new(f);
		ClassFile::parse(&mut reader)
	}
	
	fn write(class: ClassFile, dir: &String) -> Result<()> {
		let f = OpenOptions::new().write(true).open(dir).unwrap();
		let mut writer = BufWriter::new(f);
		class.write(&mut writer)
	}
	
    fn print_read(dir: &String) -> Result<ClassFile> {
	    let class = read(dir)?;
		println!("{:#x?}", class);
	    Ok(class)
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
		/*walk("classes/benchmarking/", &|entry| {
			let path = entry.path();
			if path.is_file() {
				let extension = path.extension().unwrap().to_str().unwrap();
				if extension == "class" {
					read(path.into_os_string().to_str().unwrap()).unwrap();
				}
			}
			Ok(())
		})?;*/
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
					let dir = path.into_os_string().into_string().unwrap();
					let class = print_read(&dir).unwrap();
					write(class, &dir)?;
				}
			}
			Ok(())
		})?;
		Ok(())
	}
}
