use crate::ast::{Insn, LabelInsn};

#[derive(Clone, PartialEq)]
pub struct InsnList<'u> {
	insns: Vec<Insn<'u>>,
	labels: Vec<Box<LabelInsn>>,
	available_labels: Vec<&'u Box<LabelInsn>>
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
	fn new_label<'a>(&'a mut self) -> &'a Box<LabelInsn> {
		if let Some(x) = self.available_labels.pop() {
			return x
		}
		let label = Box::new(LabelInsn::new(self.labels.len()));
		let borrow: &'a Box<LabelInsn> = &label;
		self.labels.push(label);
		borrow
	}
}

impl <'u> InsnList<'u> {
	fn release_label(&'u mut self, label: &'u Box<LabelInsn>) {
		self.available_labels.push(label)
	}
}
