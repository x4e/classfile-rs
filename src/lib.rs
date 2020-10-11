mod constantpool;

use std::io::{Seek, Read, Write};

mod version;
mod classfile;
mod access;


trait Serializable {
	fn parse<R: Seek + Read>(rdr: &mut R) -> Self;
	fn write<W: Seek + Write>(&self, wtr: &mut W);
}

#[cfg(test)]
mod tests {
	use std::fs::File;
	use std::io::{BufReader, BufWriter};
	use crate::classfile::ClassFile;
	use crate::Serializable;
	
	#[test]
    fn it_works() {
		// Read
		let class = {
			let dir = "Test.class";
			let f = File::open(dir).unwrap();
			let mut reader = BufReader::new(f);
			//let mut cursor = Cursor::new(f);
			ClassFile::parse(&mut reader)
		};
		
		println!("{:#?}", class);
		
		// Write
		{
			let dir = "TestOut.class";
			let f = File::create(dir).unwrap();
			let mut writer = BufWriter::new(f);
			//let mut cursor = Cursor::new(writer);
			class.write(&mut writer);
		}
    }
}
