use derive_more::Constructor;
use std::collections::HashMap;

#[derive(Clone, PartialEq, Eq)]
pub enum Type {
	Reference(Option<String>), // If None then the reference refers to no particular class
	Primitive(Primitive)
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Primitive {
	Boolean,
	Byte,
	Char,
	Short,
	Int,
	Long,
	Float,
	Double
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum OpType {
	Reference,
	Int,
	Float,
	Double,
	Long
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum ReturnType {
	Void,
	Reference,
	Boolean,
	Byte,
	Char,
	Short,
	Int,
	Long,
	Float,
	Double
}

#[derive(Constructor, Copy, Clone, PartialEq, Eq)]
pub struct LabelInsn {
	id: u32 // unique identifier
}

#[derive(Constructor, Copy, Clone, PartialEq, Eq)]
pub struct ArrayLoadInsn {
	pub kind: OpType,
}

#[derive(Constructor, Copy, Clone, PartialEq, Eq)]
pub struct ArrayStoreInsn {
	pub kind: OpType,
}

#[derive(Constructor, Clone, PartialEq)]
pub struct LdcInsn {
	pub constant: LdcType
}

#[derive(Clone, PartialEq)]
pub enum LdcType {
	Null,
	String(String),
	Int(i32),
	Float(f32),
	Long(i64),
	Double(f64)
}

/// Loads a value from the local array slot
#[derive(Constructor, Copy, Clone, PartialEq, Eq)]
pub struct LocalLoadInsn {
	pub kind: OpType,
	pub index: u16 // u8 with normal load, u16 with wide load
}

#[derive(Constructor, Copy, Clone, PartialEq, Eq)]
pub struct LocalStoreInsn {
	pub kind: OpType,
	pub index: u16 // u8 with normal load, u16 with wide load
}

#[derive(Constructor, Clone, PartialEq, Eq)]
pub struct NewArrayInsn {
	pub kind: Type,
}

#[derive(Constructor, Copy, Clone, PartialEq, Eq)]
pub struct ReturnInsn {
	pub kind: ReturnType
}

#[derive(Constructor, Copy, Clone, PartialEq, Eq)]
pub struct ArrayLengthInsn {}

#[derive(Constructor, Copy, Clone, PartialEq, Eq)]
pub struct ThrowInsn {}

#[derive(Constructor, Clone, PartialEq, Eq)]
pub struct CheckCastInsn {
	pub kind: String
}

#[derive(Constructor, Copy, Clone, PartialEq, Eq)]
pub struct ConvertInsn {
	pub from: Primitive,
	pub to: Primitive
}

#[derive(Constructor, Copy, Clone, PartialEq, Eq)]
pub struct AddInsn {
	pub kind: Primitive
}

#[derive(Constructor, Copy, Clone, PartialEq, Eq)]
pub struct CompareInsn {
	pub kind: Primitive
}

#[derive(Constructor, Copy, Clone, PartialEq, Eq)]
pub struct DivideInsn {
	pub kind: Primitive
}

#[derive(Constructor, Copy, Clone, PartialEq, Eq)]
pub struct MultiplyInsn {
	pub kind: Primitive
}

#[derive(Constructor, Copy, Clone, PartialEq, Eq)]
pub struct NegateInsn {
	pub kind: Primitive
}

#[derive(Constructor, Copy, Clone, PartialEq, Eq)]
pub struct RemainderInsn {
	pub kind: Primitive
}

#[derive(Constructor, Copy, Clone, PartialEq, Eq)]
pub struct SubtractInsn {
	pub kind: Primitive
}

#[derive(Constructor, Copy, Clone, PartialEq, Eq)]
pub struct AndInsn {
	pub kind: Primitive
}

#[derive(Constructor, Copy, Clone, PartialEq, Eq)]
pub struct OrInsn {
	pub kind: Primitive
}

#[derive(Constructor, Copy, Clone, PartialEq, Eq)]
pub struct XorInsn {
	pub kind: Primitive
}

#[derive(Constructor, Copy, Clone, PartialEq, Eq)]
pub struct ShiftLeft {}

/// Arithmetically shift right
#[derive(Constructor, Copy, Clone, PartialEq, Eq)]
pub struct ShiftRight {}

#[derive(Constructor, Copy, Clone, PartialEq, Eq)]
pub struct LogicalShiftRight {}

/// duplicates the value at the top of the stack
#[derive(Constructor, Copy, Clone, PartialEq, Eq)]
pub struct DupInsn {
	/// The number of items to duplicate
	pub num: u8,
	/// The number of slots down to insert it
	pub down: u8
}

#[derive(Constructor, Copy, Clone, PartialEq, Eq)]
pub struct Pop {
	/// The number of items to pop
	pub num: u8,
}

#[derive(Constructor, Clone, PartialEq, Eq)]
pub struct GetFieldInsn {
	/// Is this field an instance or static field?
	pub instance: bool,
	/// The declaring class
	pub class: String,
	/// The field name
	pub name: String,
	/// The field descriptor
	pub descriptor: String,
}

#[derive(Constructor, Clone, PartialEq, Eq)]
pub struct PutFieldInsn {
	/// Is this field an instance or static field?
	pub instance: bool,
	/// The declaring class
	pub class: String,
	/// The field name
	pub name: String,
	/// The field descriptor
	pub descriptor: String,
}

/// Unconditional Jump
#[derive(Constructor, Copy, Clone, PartialEq, Eq)]
pub struct JumpInsn<'u> {
	pub jump_to: &'u LabelInsn
}

#[derive(Constructor, Copy, Clone, PartialEq, Eq)]
pub struct ConditionalJumpInsn<'u> {
	pub condition: JumpCondition,
	pub jump_to: &'u LabelInsn
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum JumpCondition {
	IsNull,
	NotNull,
	ReferencesEqual,
	ReferencesNotEqual,
	IntsEq,
	IntsNotEq,
	IntLessThan,
	IntLessThanOrEq,
	IntGreaterThan,
	IntGreaterThanOrEq,
	IntEqZero,
	IntNotEqZero,
	IntLessThanZero,
	IntLessThanOrEqZero,
	IntGreaterThanZero,
	IntGreaterThanOrEqZero,
}

#[derive(Constructor, Copy, Clone, PartialEq, Eq)]
pub struct IncrementIntInsn {
	/// Index of the local variable
	pub index: u16,
	/// Amount to increment by
	pub amount: i16
}

#[derive(Constructor, Clone, PartialEq, Eq)]
pub struct InstanceOfInsn {
	pub class: String
}

#[derive(Constructor, Clone, PartialEq, Eq)]
pub struct InvokeDynamicInsn {
	pub name: String,
	pub descriptor: String,
	pub bootstrap_type: BootstrapMethodType,
	pub bootstrap_class: String,
	pub bootstrap_method: String,
	pub bootstrap_descriptor: String
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum BootstrapMethodType {
	InvokeStatic,
	NewInvokeSpecial
}

#[derive(Constructor, Clone, PartialEq, Eq)]
pub struct InvokeInsn {
	pub kind: InvokeType,
	pub class: String,
	pub name: String,
	pub descriptor: String
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum InvokeType {
	Instance,
	Static,
	Interface,
	Special
}

#[derive(Constructor, Clone, PartialEq, Eq)]
pub struct LookupSwitchInsn<'u> {
	pub default: &'u LabelInsn,
	pub cases: HashMap<i32, &'u LabelInsn>
}

#[derive(Constructor, Clone, PartialEq, Eq)]
pub struct TableSwitchInsn<'u> {
	pub default: &'u LabelInsn,
	pub(crate) low: i32,
	pub(crate) cases: Vec<&'u LabelInsn>
}

impl<'u> TableSwitchInsn<'u> {
	#[allow(dead_code)]
	fn get(&self, case: i32) -> Option<&'u LabelInsn> {
		if let Some(x) = self.cases.get((case - self.low) as usize) {
			Some(*x)
		} else {
			None
		}
	}
}

#[derive(Constructor, Copy, Clone, PartialEq, Eq)]
pub struct MonitorEnter {}

#[derive(Constructor, Copy, Clone, PartialEq, Eq)]
pub struct MonitorExit {}

/// New multi dimensional object array
#[derive(Constructor, Clone, PartialEq, Eq)]
pub struct MultiNewArray {
	pub kind: String,
	pub dimensions: u8
}

#[derive(Constructor, Clone, PartialEq, Eq)]
pub struct NewObject {
	pub kind: String
}

#[derive(Constructor, Copy, Clone, PartialEq, Eq)]
pub struct Nop {}

#[derive(Constructor, Copy, Clone, PartialEq, Eq)]
pub struct Swap {}

#[derive(Clone, PartialEq)]
pub enum Insn {
	Label(LabelInsn),
	ArrayLoad(ArrayLoadInsn),
	ArrayStore(ArrayStoreInsn),
	Ldc(LdcInsn),
	LocalLoad(LocalLoadInsn),
	LocalStore(LocalStoreInsn),
	NewArray(NewArrayInsn),
	Return(ReturnInsn),
	ArrayLength(ArrayLengthInsn),
	Throw(ThrowInsn),
	CheckCast(CheckCastInsn),
}
