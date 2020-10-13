use std::time::Instant;
use std::fs::File;
use std::io::{BufReader};
use std::env;

use classfile::classfile::ClassFile;

fn main() {
	let args: Vec<String> = env::args().collect();
	
	if let Some(file) = args.get(1) {
		// Read
		let start = Instant::now();
		let class = {
			let f = File::open(file).unwrap();
			let mut reader = BufReader::new(f);
			ClassFile::parse(&mut reader)
		};
		
		let elapsed = start.elapsed();
		println!("{:#?}", class);
		println!("Finished parsing {} in {:#?}", file, elapsed);
	} else {
		panic!("Please provide a file to dissasemble");
	}
}
