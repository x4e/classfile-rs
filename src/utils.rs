use std::io::Read;

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
