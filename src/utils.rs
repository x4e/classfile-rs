use std::io::Read;

pub trait VecUtils <T> {
	/// Overwrites the given index with the given item and returns the previous item if successful
	/// May return null if the index is out of bounds
	fn replace(&mut self, index: usize, item: T) -> Option<T>;
	
	fn find_first<F>(&self, op: F) -> Option<usize>
	where
		F: FnMut(&T) -> bool;
}

impl <T> VecUtils<T> for Vec<T> {
	fn replace<T>(&mut self, index: usize, item: T) -> Option<T> {
		let v = &mut **self;
		if index >= self.len() {
			return None;
		}
		Some(std::mem::replace(&mut v[index], item))
	}
	
	fn find_first<F>(&self, op: F) -> Option<usize> where
		F: FnMut(&T) -> bool {
		let mut i = 0usize;
		for attr in self.iter() {
			if op(attr) {
				return Some(i);
			}
			i += 1;
		}
		None
	}
}

pub trait ReadUtils: Read {
	#[inline]
	fn read_nbytes(&mut self, nbytes: usize) -> std::io::Result<Vec<u8>> {
		let mut buf = vec![0u8; nbytes];
		self.read_exact(&mut buf)?;
		Ok(buf)
	}
}
impl<W: Read + ?Sized> ReadUtils for W {}
