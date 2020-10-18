use crate::ast::{Insn, LabelInsn};

#[derive(Clone, Debug, PartialEq)]
pub struct InsnList {
	pub insns: Vec<Insn>,
	pub(crate) labels: u32
}

#[allow(dead_code)]
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
