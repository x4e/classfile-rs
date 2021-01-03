use crate::ast::{Insn, LabelInsn};
use std::fmt::{Debug, Formatter,};
use std::slice::Iter;

#[derive(Clone, PartialEq)]
pub struct InsnList {
	pub insns: Vec<Insn>,
	pub(crate) labels: u32
}

#[allow(dead_code)]
impl InsnList {
	pub fn new() -> Self {
		InsnList {
			insns: Vec::new(),
			labels: 0
		}
	}
	
	pub fn with_capacity(capacity: usize) -> Self {
		InsnList {
			insns: Vec::with_capacity(capacity),
			labels: 0
		}
	}
	
	/// The given label will be valid for the lifetime of this list
	pub fn new_label(&mut self) -> LabelInsn {
		let id = self.labels;
		self.labels += 1;
		LabelInsn::new(id)
	}
	
	pub fn iter(&self) -> Iter<'_, Insn> {
		self.insns.iter()
	}
	
	pub fn len(&self) -> usize {
		self.insns.len()
	}
}


impl Debug for InsnList {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		f.debug_list()
			.entries(&self.insns)
			.finish()
	}
}
