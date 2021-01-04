use crate::types::Type;
use derive_more::Constructor;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use enum_display_derive::DisplayDebug;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PrimitiveType {
	Boolean,
	Byte,
	Char,
	Short,
	Int,
	Long,
	Float,
	Double
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum OpType {
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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum IntegerType {
	Int,
	Long
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct LabelInsn {
	/// unique identifier
	pub(crate) id: u32
}

impl LabelInsn {
	pub(crate) fn new(id: u32) -> Self {
		LabelInsn { id }
	}
}

#[derive(Constructor, Clone, Debug, PartialEq, Eq)]
pub struct ArrayLoadInsn {
	pub kind: Type,
}

#[derive(Constructor, Clone, Debug, PartialEq, Eq)]
pub struct ArrayStoreInsn {
	pub kind: Type,
}

#[derive(Constructor, Clone, Debug, PartialEq)]
pub struct LdcInsn {
	pub constant: LdcType
}

#[derive(Clone, Debug, PartialEq)]
pub enum LdcType {
	Null,
	String(String),
	Int(i32),
	Float(f32),
	Long(i64),
	Double(f64),
	Class(String),
	/// Method Descriptor (java.lang.invoke.MethodType)
	MethodType(String),
	/// TODO: Method Handle (java.lang.invoke.MethodHandle)
	MethodHandle(),
	// TODO: Constant_Dynamic
	Dynamic()
}

/// Loads a value from the local array slot
#[derive(Constructor, Copy, Clone, Debug, PartialEq, Eq)]
pub struct LocalLoadInsn {
	pub kind: OpType,
	pub index: u16 // u8 with normal load, u16 with wide load
}

#[derive(Constructor, Copy, Clone, Debug, PartialEq, Eq)]
pub struct LocalStoreInsn {
	pub kind: OpType,
	pub index: u16 // u8 with normal load, u16 with wide load
}

#[derive(Constructor, Clone, Debug, PartialEq, Eq)]
pub struct NewArrayInsn {
	pub kind: Type,
}

#[derive(Constructor, Copy, Clone, Debug, PartialEq, Eq)]
pub struct ReturnInsn {
	pub kind: ReturnType
}

#[derive(Constructor, Copy, Clone, Debug, PartialEq, Eq)]
pub struct ArrayLengthInsn {}

#[derive(Constructor, Copy, Clone, Debug, PartialEq, Eq)]
pub struct ThrowInsn {}

#[derive(Constructor, Clone, Debug, PartialEq, Eq)]
pub struct CheckCastInsn {
	pub kind: String
}

#[derive(Constructor, Copy, Clone, Debug, PartialEq, Eq)]
pub struct ConvertInsn {
	pub from: PrimitiveType,
	pub to: PrimitiveType
}

#[derive(Constructor, Copy, Clone, Debug, PartialEq, Eq)]
pub struct AddInsn {
	pub kind: PrimitiveType
}

#[derive(Constructor, Copy, Clone, Debug, PartialEq, Eq)]
pub struct CompareInsn {
	pub kind: PrimitiveType,
	/// If both values are NAN and this flag is set, 1 will be pushed. Otherwise -1 will be pushed.
	pub pos_on_nan: bool
}

#[derive(Constructor, Copy, Clone, Debug, PartialEq, Eq)]
pub struct DivideInsn {
	pub kind: PrimitiveType
}

#[derive(Constructor, Copy, Clone, Debug, PartialEq, Eq)]
pub struct MultiplyInsn {
	pub kind: PrimitiveType
}

#[derive(Constructor, Copy, Clone, Debug, PartialEq, Eq)]
pub struct NegateInsn {
	pub kind: PrimitiveType
}

#[derive(Constructor, Copy, Clone, Debug, PartialEq, Eq)]
pub struct RemainderInsn {
	pub kind: PrimitiveType
}

#[derive(Constructor, Copy, Clone, Debug, PartialEq, Eq)]
pub struct SubtractInsn {
	pub kind: PrimitiveType
}

#[derive(Constructor, Copy, Clone, Debug, PartialEq, Eq)]
pub struct AndInsn {
	pub kind: IntegerType
}

#[derive(Constructor, Copy, Clone, Debug, PartialEq, Eq)]
pub struct OrInsn {
	pub kind: IntegerType
}

#[derive(Constructor, Copy, Clone, Debug, PartialEq, Eq)]
pub struct XorInsn {
	pub kind: IntegerType
}

#[derive(Constructor, Copy, Clone, Debug, PartialEq, Eq)]
pub struct ShiftLeftInsn {
	pub kind: IntegerType
}

/// Arithmetically shift right
#[derive(Constructor, Copy, Clone, Debug, PartialEq, Eq)]
pub struct ShiftRightInsn {
	pub kind: IntegerType
}

#[derive(Constructor, Copy, Clone, Debug, PartialEq, Eq)]
pub struct LogicalShiftRightInsn {
	pub kind: IntegerType
}

/// duplicates the value at the top of the stack
#[derive(Constructor, Copy, Clone, Debug, PartialEq, Eq)]
pub struct DupInsn {
	/// The number of items to duplicate
	pub num: u8,
	/// The number of slots down to insert it
	pub down: u8
}

#[derive(Constructor, Copy, Clone, Debug, PartialEq, Eq)]
pub struct PopInsn {
	/// if false, pop a single 32bit item off the stack (not long or double)
	/// if true, pop either two 32bit items, or one 64bit item (long or double)
	pub pop_two: bool
}

#[derive(Constructor, Clone, Debug, PartialEq, Eq)]
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

#[derive(Constructor, Clone, Debug, PartialEq, Eq)]
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
#[derive(Constructor, Copy, Clone, Debug, PartialEq, Eq)]
pub struct JumpInsn {
	pub jump_to: LabelInsn
}

#[derive(Constructor, Copy, Clone, Debug, PartialEq, Eq)]
pub struct ConditionalJumpInsn {
	pub condition: JumpCondition,
	pub jump_to: LabelInsn
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum JumpCondition {
	/// The reference at the top of the stack is null
	IsNull,
	/// The reference at the top of the stack is not null
	NotNull,
	/// The two references at the top of the stack are the same
	ReferencesEqual,
	/// The two references at the top of the stack are not the same
	ReferencesNotEqual,
	/// The two ints at the top of the stack are equal
	IntsEq,
	/// The two ints at the top of the stack are not equal
	IntsNotEq,
	/// The int second on the stack is less than the int at the top of the stack
	IntsLessThan,
	/// The int second on the stack is less than or equal to the int at the top of the stack
	IntsLessThanOrEq,
	/// The int second on the stack is greater than the int at the top of the stack
	IntsGreaterThan,
	/// The int second on the stack is greater than or equal to the int at the top of the stack
	IntsGreaterThanOrEq,
	/// The int at the top of the stack is 0
	IntEqZero,
	/// The int at the top of the stack is not 0
	IntNotEqZero,
	/// The int at the top of the stack is less than 0
	IntLessThanZero,
	/// The int at the top of the stack is less than or equal to 0
	IntLessThanOrEqZero,
	/// The int at the top of the stack is greater than 0
	IntGreaterThanZero,
	/// The int at the top of the stack is less greater or equal to 0
	IntGreaterThanOrEqZero,
}

#[derive(Constructor, Copy, Clone, Debug, PartialEq, Eq)]
pub struct IncrementIntInsn {
	/// Index of the local variable
	pub index: u16,
	/// Amount to increment by
	pub amount: i16
}

#[derive(Constructor, Clone, Debug, PartialEq, Eq)]
pub struct InstanceOfInsn {
	pub class: String
}

#[derive(Constructor, Clone, Debug, PartialEq)]
pub struct InvokeDynamicInsn {
	pub name: String,
	pub descriptor: String,
	pub bootstrap_type: BootstrapMethodType,
	pub bootstrap_class: String,
	pub bootstrap_method: String,
	pub bootstrap_descriptor: String,
	pub bootstrap_arguments: Vec<BootstrapArgument>
}

#[derive(Clone, Debug, PartialEq)]
pub enum BootstrapArgument {
	Int(i32),
	Float(f32),
	Long(i64),
	Double(f64),
	Class(String)
	// TODO: Continue. Do we have to do this for every constant type? Spec seems to suggest so
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum BootstrapMethodType {
	InvokeStatic,
	NewInvokeSpecial
}

#[derive(Constructor, Clone, Debug, PartialEq, Eq)]
pub struct InvokeInsn {
	pub kind: InvokeType,
	pub class: String,
	pub name: String,
	pub descriptor: String,
	pub interface_method: bool
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum InvokeType {
	Instance,
	Static,
	Interface,
	Special
}

#[derive(Constructor, Clone, PartialEq, Eq)]
pub struct LookupSwitchInsn {
	pub default: LabelInsn,
	pub(crate) cases: HashMap<i32, LabelInsn>
}

impl LookupSwitchInsn {
	pub fn get(&self, case: i32) -> Option<LabelInsn> {
		self.cases.get(&case).cloned()
	}
}

impl Debug for LookupSwitchInsn {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		struct DebugCases<'u> {
			tbl: &'u LookupSwitchInsn
		}
		impl <'u> Debug for DebugCases<'u> {
			fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
				let mut map = f.debug_map();
				map.entry(&"default", &self.tbl.default);
				for (index, case) in self.tbl.cases.iter() {
					map.entry(&index, case);
				}
				map.finish()
			}
		}
		
		f.debug_struct("LookupSwitchInsn")
			.field("cases", &DebugCases{ tbl: &self })
			.finish()
	}
}

#[derive(Constructor, Clone, PartialEq, Eq)]
pub struct TableSwitchInsn {
	pub default: LabelInsn,
	pub(crate) low: i32,
	pub(crate) cases: Vec<LabelInsn>
}

impl TableSwitchInsn {
	#[allow(dead_code)]
	pub fn get(&self, case: i32) -> Option<LabelInsn> {
		if let Some(x) = self.cases.get((case - self.low) as usize) {
			Some(x.clone())
		} else {
			None
		}
	}
}

impl Debug for TableSwitchInsn {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		struct DebugCases<'u> {
			tbl: &'u TableSwitchInsn
		}
		impl <'u> Debug for DebugCases<'u> {
			fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
				let mut map = f.debug_map();
				map.entry(&"default", &self.tbl.default);
				let mut index = 0;
				for case in self.tbl.cases.iter() {
					map.entry(&(index + self.tbl.low), case);
					index += 1;
				}
				map.finish()
			}
		}
		
		f.debug_struct("TableSwitchInsn")
			.field("cases", &DebugCases{ tbl: &self })
			.finish()
	}
}

#[derive(Constructor, Copy, Clone, Debug, PartialEq, Eq)]
pub struct MonitorEnterInsn {}

#[derive(Constructor, Copy, Clone, Debug, PartialEq, Eq)]
pub struct MonitorExitInsn {}

/// New multi dimensional object array
#[derive(Constructor, Clone, Debug, PartialEq, Eq)]
pub struct MultiNewArrayInsn {
	pub kind: String,
	pub dimensions: u8
}

#[derive(Constructor, Clone, Debug, PartialEq, Eq)]
pub struct NewObjectInsn {
	pub kind: String
}

#[derive(Constructor, Copy, Clone, Debug, PartialEq, Eq)]
pub struct NopInsn {}

#[derive(Constructor, Copy, Clone, Debug, PartialEq, Eq)]
pub struct SwapInsn {}

/// Implementation dependent insn
#[derive(Constructor, Copy, Clone, Debug, PartialEq, Eq)]
pub struct ImpDep1Insn {}

/// Implementation dependent insn
#[derive(Constructor, Copy, Clone, Debug, PartialEq, Eq)]
pub struct ImpDep2Insn {}

/// Used by debuggers
#[derive(Constructor, Copy, Clone, Debug, PartialEq, Eq)]
pub struct BreakPointInsn {}

#[derive(Clone, PartialEq, DisplayDebug)]
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
	Convert(ConvertInsn),
	Add(AddInsn),
	Compare(CompareInsn),
	Divide(DivideInsn),
	Multiply(MultiplyInsn),
	Negate(NegateInsn),
	Remainder(RemainderInsn),
	Subtract(SubtractInsn),
	And(AndInsn),
	Or(OrInsn),
	Xor(XorInsn),
	ShiftLeft(ShiftLeftInsn),
	ShiftRight(ShiftRightInsn),
	LogicalShiftRight(LogicalShiftRightInsn),
	Dup(DupInsn),
	Pop(PopInsn),
	GetField(GetFieldInsn),
	PutField(PutFieldInsn),
	Jump(JumpInsn),
	ConditionalJump(ConditionalJumpInsn),
	IncrementInt(IncrementIntInsn),
	InstanceOf(InstanceOfInsn),
	InvokeDynamic(InvokeDynamicInsn),
	Invoke(InvokeInsn),
	LookupSwitch(LookupSwitchInsn),
	TableSwitch(TableSwitchInsn),
	MonitorEnter(MonitorEnterInsn),
	MonitorExit(MonitorExitInsn),
	MultiNewArray(MultiNewArrayInsn),
	NewObject(NewObjectInsn),
	Nop(NopInsn),
	Swap(SwapInsn),
	ImpDep1(ImpDep1Insn),
	ImpDep2(ImpDep2Insn),
	BreakPoint(BreakPointInsn)
}
