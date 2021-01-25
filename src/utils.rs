use std::io::Read;
use std::collections::HashMap;
use std::hash::Hash;

pub trait VecUtils <T> {
	/// Overwrites the given index with the given item and returns the previous item if successful
	/// May return null if the index is out of bounds
	fn replace(&mut self, index: usize, item: T) -> Option<T>;
	
	fn find_first<F>(&self, op: F) -> Option<usize>
	where
		F: FnMut(&T) -> bool;
}

impl <T> VecUtils<T> for Vec<T> {
	fn replace(&mut self, index: usize, item: T) -> Option<T> {
		if index >= self.len() {
			return None;
		}
		let v = &mut **self;
		Some(std::mem::replace(&mut v[index], item))
	}
	
	fn find_first<F>(&self, mut op: F) -> Option<usize> where
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

pub trait MapUtils <K, V> {
	/// returns true if inserted
	fn insert_if_not_present(&mut self, key: K, value: V) -> bool;
	
	fn insert_if_not_present_lazy<F>(&mut self, key: K, value: F) -> bool
		where F: FnOnce() -> V;
}

impl <K: Eq + Hash, V> MapUtils<K, V> for HashMap<K, V> {
	#[inline]
	fn insert_if_not_present(&mut self, key: K, value: V) -> bool {
		if self.get(&key).is_none() {
			self.insert(key, value);
			true
		} else {
			false
		}
	}
	
	#[inline]
	fn insert_if_not_present_lazy<F>(&mut self, key: K, value: F) -> bool where F: FnOnce() -> V {
		if self.get(&key).is_none() {
			self.insert(key, value());
			true
		} else {
			false
		}
	}
}
