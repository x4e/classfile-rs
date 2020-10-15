use crate::ast::{Insn, LabelInsn};
use std::borrow::Borrow;

#[derive(Clone, PartialEq)]
pub struct InsnList<'u> {
	insns: Vec<Insn<'u>>,
	labels: Vec<LabelInsn>,
	available_labels: Vec<&'u LabelInsn>
}

impl InsnList<'_> {
	fn new() -> Self {
		InsnList {
			insns: vec![],
			labels: vec![],
			available_labels: vec![]
		}
	}
	
	/// The givien label will be valid for the lifetime of this list
	fn new_label(&mut self) -> &LabelInsn {
		if let Some(x) = self.available_labels.pop() {
			return x
		}
		let label = LabelInsn::new(self.labels.len());
		self.labels.push(label);
		self.labels.last().unwrap()
	}
}

impl <'u> InsnList<'u> {
	fn release_label(&'u mut self, label: &'u LabelInsn) {
		self.available_labels.push(label)
	}
}
