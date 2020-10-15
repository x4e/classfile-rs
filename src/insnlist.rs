use crate::ast::{Insn, LabelInsn};

#[derive(Clone, PartialEq)]
pub struct InsnList {
	insns: Vec<Insn>,
	labels: u32
}

impl InsnList {
	fn new() -> Self {
		InsnList {
			insns: vec![],
			labels: 0
		}
	}
	
	/// The givien label will be valid for the lifetime of this list
	fn new_label(&mut self) -> LabelInsn {
		let id = self.labels;
		self.labels += 1;
		LabelInsn::new(id)
	}
}
