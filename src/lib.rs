mod constantpool;

use std::io::{Seek, Read, Write};

mod version;
mod classfile;
mod access;
mod field;
mod attributes;


trait Serializable {
	fn parse<R: Seek + Read>(rdr: &mut R) -> Self;
	fn write<W: Seek + Write>(&self, wtr: &mut W);
}

#[cfg(test)]
mod tests {
	use std::fs::File;
	use std::io::{BufReader, BufWriter};
	use std::time::Instant;
	use crate::classfile::ClassFile;
	use crate::Serializable;
	
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
		println!("{:#?}", class);
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
