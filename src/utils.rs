use std::io::Read;
use std::convert::TryFrom;

pub fn mut_retain<T, F>(this: &mut Vec<T>, mut f: F)
	where
		F: FnMut(&mut T) -> bool,
{
	let len = this.len();
	let mut del = 0;
	{
		let v = &mut **this;
		
		for i in 0..len {
			if !f(&mut v[i]) {
				del += 1;
			} else if del > 0 {
				v.swap(i - del, i);
			}
		}
	}
	if del > 0 {
		this.truncate(len - del);
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


/// Rust floats do not support Eq and Hash
/// This is because its just too hard to correctly compare floats
/// For our purpose however we dont care too much about equality and more about not (?) equality
/// In the end if two of the same floats are not compared equal to each other then we just make
/// two constant pool entries and who cares
/// Because of this we will store the float as an integer and let rust do integer comparisons on it
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct CustomFloat {
	inner: u32
}
impl CustomFloat {
	pub fn new(inner: f32) -> Self {
		CustomFloat {
			inner: unsafe { std::mem::transmute(inner) }
		}
	}
}
impl Into<f32> for CustomFloat {
	fn into(self) -> f32 {
		unsafe { std::mem::transmute(self.inner) }
	}
}
impl From<f32> for CustomFloat {
	fn from(x: f32) -> Self {
		CustomFloat::new(x)
	}
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct CustomDouble {
	inner: u64
}
impl CustomDouble {
	pub fn new(inner: f64) -> Self {
		CustomDouble {
			inner: unsafe { std::mem::transmute(inner) }
		}
	}
}
impl Into<f64> for CustomDouble {
	fn into(self) -> f64 {
		unsafe { std::mem::transmute(self.inner) }
	}
}
impl From<f64> for CustomDouble {
	fn from(x: f64) -> Self {
		CustomDouble::new(x)
	}
}

