use std::time::Instant;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::env;

use classfile::classfile::ClassFile;

fn main() {
	let args: Vec<String> = env::args().collect();
	
	if let Some(file) = args.get(1) {
		if file == "-h" {
			print_usage();
			return;
		}
		
		// Read
		let start = Instant::now();
		let class = {
			let f = File::open(file).unwrap();
			let mut reader = BufReader::new(f);
			ClassFile::parse(&mut reader)
		};
		
		let elapsed = start.elapsed();
		println!("{:#x?}", class);
		println!("Finished parsing {} in {:#?}", file, elapsed);
		
		// If the user has provided an output file we will write there
		if let Ok(class) = class {
			if let Some(file) = args.get(2) {
				let f = File::create(file).unwrap();
				let mut writer = BufWriter::new(f);
				class.write(&mut writer).unwrap();
			}
		}
	} else {
		print_usage();
	}
}

fn print_usage() {
	eprintln!("Usage: ./dissasembler classFileIn.class (classFileOut.class)");
}
