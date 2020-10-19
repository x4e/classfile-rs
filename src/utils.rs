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
