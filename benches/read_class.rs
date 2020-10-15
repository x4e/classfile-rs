use criterion::{criterion_group, criterion_main, Criterion};
use classfile::classfile::ClassFile;
use std::fs::File;
use std::io::BufReader;

fn read(mut reader: BufReader<File>) -> classfile::error::Result<ClassFile> {
	ClassFile::parse(&mut reader)
}

fn read_class_bench(c: &mut Criterion) {
	c.bench_function("read_class", |b|
		b.iter_with_setup(
			|| {
				let dir = "Class.class";
				let f = File::open(dir).unwrap();
				BufReader::new(f)
			},
			|reader| read(reader)
		)
	);
}

criterion_group!(benches, read_class_bench);
criterion_main!(benches);
