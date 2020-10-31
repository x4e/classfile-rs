use criterion::{criterion_group, criterion_main, Criterion, Throughput, BatchSize, BenchmarkId};
use classfile::classfile::ClassFile;
use std::io::{Cursor};
use std::fs;

fn read_class_bench(c: &mut Criterion) {
	let mut group = c.benchmark_group("read_class");
	
	for entry in fs::read_dir("classes/benchmarking").unwrap() {
		let entry = entry.unwrap();
		let path = entry.path();
		if path.is_file() {
			if let Some(ex) = path.extension() {
				if let Some(ex) = ex.to_str() {
					let ex = ex.to_string();
					if ex == "class" {
						let bytes: Vec<u8> = fs::read(path).unwrap();
						group.throughput(Throughput::Bytes(bytes.len() as u64));
						group.bench_with_input(BenchmarkId::from_parameter(entry.file_name().into_string().unwrap()), &bytes, |b, bytes| {
							b.iter_batched(|| Cursor::new(bytes), | mut slice |{
								ClassFile::parse(&mut slice)
							}, BatchSize::SmallInput);
						});
					}
				}
			}
		}
	}
}

criterion_group!(benches, read_class_bench);
criterion_main!(benches);
