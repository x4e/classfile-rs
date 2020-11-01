use crate::attributes::{Attribute, AttributeSource, Attributes};
use crate::constantpool::{ConstantPool, ConstantType, CPIndex};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Seek, Write};
use crate::version::ClassVersion;
use crate::error::{Result, ParserError};
use crate::ast::*;
use crate::insnlist::InsnList;
use crate::utils::ReadUtils;
use std::collections::{HashMap};
use std::mem;

#[derive(Clone, Debug, PartialEq)]
pub struct CodeAttribute {
	pub max_stack: u16,
	pub max_locals: u16,
	pub code: InsnList,
	pub exceptions: Vec<ExceptionHandler>,
	pub attributes: Vec<Attribute>
}

impl CodeAttribute {
	pub fn parse(version: &ClassVersion, constant_pool: &ConstantPool, buf: Vec<u8>) -> Result<Self> {
		let mut slice = buf.as_slice();
		let max_stack = slice.read_u16::<BigEndian>()?;
		let max_locals = slice.read_u16::<BigEndian>()?;
		let code_length = slice.read_u32::<BigEndian>()?;
		let mut code: Vec<u8> = vec![0; code_length as usize];
		slice.read_exact(&mut code)?;
		let code = InsnParser::parse_insns(constant_pool, code.as_slice(), code_length)?;
		let num_exceptions = slice.read_u16::<BigEndian>()?;
		let mut exceptions: Vec<ExceptionHandler> = Vec::with_capacity(num_exceptions as usize);
		for _ in 0..num_exceptions {
			exceptions.push(ExceptionHandler::parse(constant_pool, &mut slice)?);
		}
		let attributes = Attributes::parse(&mut slice, AttributeSource::Code, version, constant_pool)?;
		
		Ok(CodeAttribute {
			max_stack,
			max_locals,
			code,
			exceptions,
			attributes
		})
	}
	
	pub fn write<T: Seek + Write>(&self, wtr: &mut T, _constant_pool: &ConstantPool) -> Result<()> {
		wtr.write_u16::<BigEndian>(0)?; // write name
		wtr.write_u32::<BigEndian>(2)?; // length
		wtr.write_u16::<BigEndian>(0)?; // cp ref
		Ok(())
	}
}


#[derive(Clone, Debug, PartialEq)]
pub struct ExceptionHandler {
	pub start_pc: u16,
	pub end_pc: u16,
	pub handler_pc: u16,
	pub catch_type: Option<String>
}

impl ExceptionHandler {
	pub fn parse(constant_pool: &ConstantPool, buf: &mut &[u8]) -> Result<Self> {
		let start_pc = buf.read_u16::<BigEndian>()?;
		let end_pc = buf.read_u16::<BigEndian>()?;
		let handler_pc = buf.read_u16::<BigEndian>()?;
		let catch_index = buf.read_u16::<BigEndian>()?;
		let catch_type = if catch_index > 0 {
			Some(constant_pool.utf8(constant_pool.class(catch_index)?.name_index)?.str.clone())
		} else {
			None
		};
		
		Ok(ExceptionHandler {
			start_pc,
			end_pc,
			handler_pc,
			catch_type
		})
	}
	
	pub fn write<T: Seek + Write>(&self, wtr: &mut T, _constant_pool: &ConstantPool) -> Result<()> {
		wtr.write_u16::<BigEndian>(self.start_pc)?;
		wtr.write_u16::<BigEndian>(self.end_pc)?;
		wtr.write_u16::<BigEndian>(self.handler_pc)?;
		wtr.write_u16::<BigEndian>(0)?; // catch type cp ref
		Ok(())
	}
}

struct InsnParser {}
#[allow(unused_variables)]
#[allow(dead_code)]
impl InsnParser {
	const AALOAD: u8 = 0x32;
	const AASTORE: u8 = 0x53;
	const ACONST_NULL: u8 = 0x01;
	const ALOAD: u8 = 0x19;
	const ALOAD_0: u8 = 0x2A;
	const ALOAD_1: u8 = 0x2B;
	const ALOAD_2: u8 = 0x2C;
	const ALOAD_3: u8 = 0x2D;
	const ANEWARRAY: u8 = 0xBD;
	const ARETURN: u8 = 0xB0;
	const ARRAYLENGTH: u8 = 0xBE;
	const ASTORE: u8 = 0x3A;
	const ASTORE_0: u8 = 0x4B;
	const ASTORE_1: u8 = 0x4C;
	const ASTORE_2: u8 = 0x4D;
	const ASTORE_3: u8 = 0x4E;
	const ATHROW: u8 = 0xBF;
	const BALOAD: u8 = 0x33;
	const BASTORE: u8 = 0x54;
	const BIPUSH: u8 = 0x10;
	const BREAKPOINT: u8 = 0xCA;
	const CALOAD: u8 = 0x34;
	const CASTORE: u8 = 0x55;
	const CHECKCAST: u8 = 0xC0;
	const D2F: u8 = 0x90;
	const D2I: u8 = 0x8E;
	const D2L: u8 = 0x8F;
	const DADD: u8 = 0x63;
	const DALOAD: u8 = 0x31;
	const DASTORE: u8 = 0x52;
	const DCMPG: u8 = 0x98;
	const DCMPL: u8 = 0x97;
	const DCONST_0: u8 = 0x0E;
	const DCONST_1: u8 = 0x0F;
	const DDIV: u8 = 0x6F;
	const DLOAD: u8 = 0x18;
	const DLOAD_0: u8 = 0x26;
	const DLOAD_1: u8 = 0x27;
	const DLOAD_2: u8 = 0x28;
	const DLOAD_3: u8 = 0x29;
	const DMUL: u8 = 0x6B;
	const DNEG: u8 = 0x77;
	const DREM: u8 = 0x73;
	const DRETURN: u8 = 0xAF;
	const DSTORE: u8 = 0x39;
	const DSTORE_0: u8 = 0x47;
	const DSTORE_1: u8 = 0x48;
	const DSTORE_2: u8 = 0x49;
	const DSTORE_3: u8 = 0x4A;
	const DSUB: u8 = 0x67;
	const DUP: u8 = 0x59;
	const DUP_X1: u8 = 0x5A;
	const DUP_X2: u8 = 0x5B;
	const DUP2: u8 = 0x5C;
	const DUP2_X1: u8 = 0x5D;
	const DUP2_X2: u8 = 0x5E;
	const F2D: u8 = 0x8D;
	const F2I: u8 = 0x8B;
	const F2L: u8 = 0x8C;
	const FADD: u8 = 0x62;
	const FALOAD: u8 = 0x30;
	const FASTORE: u8 = 0x51;
	const FCMPG: u8 = 0x96;
	const FCMPL: u8 = 0x95;
	const FCONST_0: u8 = 0x0B;
	const FCONST_1: u8 = 0x0C;
	const FCONST_2: u8 = 0x0D;
	const FDIV: u8 = 0x6E;
	const FLOAD: u8 = 0x17;
	const FLOAD_0: u8 = 0x22;
	const FLOAD_1: u8 = 0x23;
	const FLOAD_2: u8 = 0x24;
	const FLOAD_3: u8 = 0x25;
	const FMUL: u8 = 0x6A;
	const FNEG: u8 = 0x76;
	const FREM: u8 = 0x72;
	const FRETURN: u8 = 0xAE;
	const FSTORE: u8 = 0x38;
	const FSTORE_0: u8 = 0x43;
	const FSTORE_1: u8 = 0x44;
	const FSTORE_2: u8 = 0x45;
	const FSTORE_3: u8 = 0x46;
	const FSUB: u8 = 0x66;
	const GETFIELD: u8 = 0xB4;
	const GETSTATIC: u8 = 0xB2;
	const GOTO: u8 = 0xA7;
	const GOTO_W: u8 = 0xC8;
	const I2B: u8 = 0x91;
	const I2C: u8 = 0x92;
	const I2D: u8 = 0x87;
	const I2F: u8 = 0x86;
	const I2L: u8 = 0x85;
	const I2S: u8 = 0x93;
	const IADD: u8 = 0x60;
	const IALOAD: u8 = 0x2E;
	const IAND: u8 = 0x7E;
	const IASTORE: u8 = 0x4F;
	const ICONST_M1: u8 = 0x02;
	const ICONST_0: u8 = 0x03;
	const ICONST_1: u8 = 0x04;
	const ICONST_2: u8 = 0x05;
	const ICONST_3: u8 = 0x06;
	const ICONST_4: u8 = 0x07;
	const ICONST_5: u8 = 0x08;
	const IDIV: u8 = 0x6C;
	const IF_ACMPEQ: u8 = 0xA5;
	const IF_ACMPNE: u8 = 0xA6;
	const IF_ICMPEQ: u8 = 0x9F;
	const IF_ICMPGE: u8 = 0xA2;
	const IF_ICMPGT: u8 = 0xA3;
	const IF_ICMPLE: u8 = 0xA4;
	const IF_ICMPLT: u8 = 0xA1;
	const IF_ICMPNE: u8 = 0xA0;
	const IFEQ: u8 = 0x99;
	const IFGE: u8 = 0x9C;
	const IFGT: u8 = 0x9D;
	const IFLE: u8 = 0x9E;
	const IFLT: u8 = 0x9B;
	const IFNE: u8 = 0x9A;
	const IFNONNULL: u8 = 0xC7;
	const IFNULL: u8 = 0xC6;
	const IINC: u8 = 0x84;
	const ILOAD: u8 = 0x15;
	const ILOAD_0: u8 = 0x1A;
	const ILOAD_1: u8 = 0x1B;
	const ILOAD_2: u8 = 0x1C;
	const ILOAD_3: u8 = 0x1D;
	const IMPDEP1: u8 = 0xFE;
	const IMPDEP2: u8 = 0xFF;
	const IMUL: u8 = 0x68;
	const INEG: u8 = 0x74;
	const INSTANCEOF: u8 = 0xC1;
	const INVOKEDYNAMIC: u8 = 0xBA;
	const INVOKEINTERFACE: u8 = 0xB9;
	const INVOKESPECIAL: u8 = 0xB7;
	const INVOKESTATIC: u8 = 0xB8;
	const INVOKEVIRTUAL: u8 = 0xB6;
	const IOR: u8 = 0x80;
	const IREM: u8 = 0x70;
	const IRETURN: u8 = 0xAC;
	const ISHL: u8 = 0x78;
	const ISHR: u8 = 0x7A;
	const ISTORE: u8 = 0x36;
	const ISTORE_0: u8 = 0x3B;
	const ISTORE_1: u8 = 0x3C;
	const ISTORE_2: u8 = 0x3D;
	const ISTORE_3: u8 = 0x3E;
	const ISUB: u8 = 0x64;
	const IUSHR: u8 = 0x7C;
	const IXOR: u8 = 0x82;
	const JSR: u8 = 0xA8;
	const JSR_W: u8 = 0xC9;
	const L2D: u8 = 0x8A;
	const L2F: u8 = 0x89;
	const L2I: u8 = 0x88;
	const LADD: u8 = 0x61;
	const LALOAD: u8 = 0x2F;
	const LAND: u8 = 0x7F;
	const LASTORE: u8 = 0x50;
	const LCMP: u8 = 0x94;
	const LCONST_0: u8 = 0x09;
	const LCONST_1: u8 = 0x0A;
	const LDC: u8 = 0x12;
	const LDC_W: u8 = 0x13;
	const LDC2_W: u8 = 0x14;
	const LDIV: u8 = 0x6D;
	const LLOAD: u8 = 0x16;
	const LLOAD_0: u8 = 0x1E;
	const LLOAD_1: u8 = 0x1F;
	const LLOAD_2: u8 = 0x20;
	const LLOAD_3: u8 = 0x21;
	const LMUL: u8 = 0x69;
	const LNEG: u8 = 0x75;
	const LOOKUPSWITCH: u8 = 0xAB;
	const LOR: u8 = 0x81;
	const LREM: u8 = 0x71;
	const LRETURN: u8 = 0xAD;
	const LSHL: u8 = 0x79;
	const LSHR: u8 = 0x7B;
	const LSTORE: u8 = 0x37;
	const LSTORE_0: u8 = 0x3F;
	const LSTORE_1: u8 = 0x40;
	const LSTORE_2: u8 = 0x41;
	const LSTORE_3: u8 = 0x42;
	const LSUB: u8 = 0x65;
	const LUSHR: u8 = 0x7D;
	const LXOR: u8 = 0x83;
	const MONITORENTER: u8 = 0xC2;
	const MONITOREXIT: u8 = 0xC3;
	const MULTIANEWARRAY: u8 = 0xC5;
	const NEW: u8 = 0xBB;
	const NEWARRAY: u8 = 0xBC;
	const NOP: u8 = 0x00;
	const POP: u8 = 0x57;
	const POP2: u8 = 0x58;
	const PUTFIELD: u8 = 0xB5;
	const PUTSTATIC: u8 = 0xB3;
	const RET: u8 = 0xA9;
	const RETURN: u8 = 0xB1;
	const SALOAD: u8 = 0x35;
	const SASTORE: u8 = 0x56;
	const SIPUSH: u8 = 0x11;
	const SWAP: u8 = 0x5F;
	const TABLESWITCH: u8 = 0xAA;
	const WIDE: u8 = 0xC4;
	
	fn parse_insns<'m, T: Read>(constant_pool: &ConstantPool, mut rdr: T, length: u32) -> Result<InsnList> {
		let num_insns_estimate = length as usize / 3; // conservative assumption average 3 bytes per insn
		let mut insns: Vec<Insn> = Vec::with_capacity(num_insns_estimate);
		let mut required_labels: u32 = 0;
		
		let mut pc_index_map: HashMap<u32, u32> = HashMap::with_capacity(num_insns_estimate);
		
		let mut pc: u32 = 0;
		let mut index: u32 = 0;
		while pc < length {
			let this_pc = pc;
			let opcode = rdr.read_u8()?;
			pc += 1;
			//println!("Parsing {:X?}", opcode);
			
			let insn = match opcode {
				InsnParser::AALOAD => Insn::ArrayLoad(ArrayLoadInsn::new(Type::Reference(None))),
				InsnParser::AASTORE => Insn::ArrayStore(ArrayStoreInsn::new(Type::Reference(None))),
				InsnParser::ACONST_NULL => Insn::Ldc(LdcInsn::new(LdcType::Null)),
				InsnParser::ALOAD => {
					let index = rdr.read_u8()?;
					pc += 1;
					Insn::LocalLoad(LocalLoadInsn::new(OpType::Reference, index as u16))
				},
				InsnParser::ALOAD_0 => Insn::LocalLoad(LocalLoadInsn::new(OpType::Reference, 0)),
				InsnParser::ALOAD_1 => Insn::LocalLoad(LocalLoadInsn::new(OpType::Reference, 1)),
				InsnParser::ALOAD_2 => Insn::LocalLoad(LocalLoadInsn::new(OpType::Reference, 2)),
				InsnParser::ALOAD_3 => Insn::LocalLoad(LocalLoadInsn::new(OpType::Reference, 3)),
				InsnParser::ANEWARRAY => {
					let kind = constant_pool.utf8(constant_pool.class(rdr.read_u16::<BigEndian>()?)?.name_index)?.str.clone();
					pc += 2;
					Insn::NewArray(NewArrayInsn::new(Type::Reference(Some(kind))))
				},
				InsnParser::ARETURN => Insn::Return(ReturnInsn::new(ReturnType::Reference)),
				InsnParser::ARRAYLENGTH => Insn::ArrayLength(ArrayLengthInsn::new()),
				InsnParser::ASTORE => {
					let index = rdr.read_u8()?;
					pc += 1;
					Insn::LocalStore(LocalStoreInsn::new(OpType::Reference, index as u16))
				},
				InsnParser::ASTORE_1 => Insn::LocalStore(LocalStoreInsn::new(OpType::Reference, 1)),
				InsnParser::ASTORE_2 => Insn::LocalStore(LocalStoreInsn::new(OpType::Reference, 2)),
				InsnParser::ASTORE_3 => Insn::LocalStore(LocalStoreInsn::new(OpType::Reference, 3)),
				InsnParser::ATHROW => Insn::Throw(ThrowInsn::new()),
				InsnParser::BALOAD => Insn::ArrayLoad(ArrayLoadInsn::new(Type::Primitive(PrimitiveType::Byte))),
				InsnParser::BASTORE => Insn::ArrayStore(ArrayStoreInsn::new(Type::Primitive(PrimitiveType::Byte))),
				InsnParser::BIPUSH => {
					let byte = rdr.read_i8()?;
					pc += 1;
					Insn::Ldc(LdcInsn::new(LdcType::Int(byte as i32)))
				},
				InsnParser::BREAKPOINT => Insn::BreakPoint(BreakPointInsn::new()),
				InsnParser::CALOAD => Insn::ArrayLoad(ArrayLoadInsn::new(Type::Primitive(PrimitiveType::Char))),
				InsnParser::CASTORE => Insn::ArrayStore(ArrayStoreInsn::new(Type::Primitive(PrimitiveType::Char))),
				InsnParser::CHECKCAST => {
					let kind = constant_pool.utf8(constant_pool.class(rdr.read_u16::<BigEndian>()?)?.name_index)?.str.clone();
					pc += 2;
					Insn::CheckCast(CheckCastInsn::new(kind))
				},
				InsnParser::D2F => Insn::Convert(ConvertInsn::new(PrimitiveType::Double, PrimitiveType::Float)),
				InsnParser::D2I => Insn::Convert(ConvertInsn::new(PrimitiveType::Double, PrimitiveType::Int)),
				InsnParser::D2L => Insn::Convert(ConvertInsn::new(PrimitiveType::Double, PrimitiveType::Long)),
				InsnParser::DADD => Insn::Add(AddInsn::new(PrimitiveType::Double)),
				InsnParser::DALOAD => Insn::ArrayLoad(ArrayLoadInsn::new(Type::Primitive(PrimitiveType::Double))),
				InsnParser::DASTORE => Insn::ArrayStore(ArrayStoreInsn::new(Type::Primitive(PrimitiveType::Double))),
				InsnParser::DCMPG => Insn::Compare(CompareInsn::new(PrimitiveType::Double, true)),
				InsnParser::DCMPL => Insn::Compare(CompareInsn::new(PrimitiveType::Double, false)),
				InsnParser::DCONST_0 => Insn::Ldc(LdcInsn::new(LdcType::Double(0f64))),
				InsnParser::DCONST_1 => Insn::Ldc(LdcInsn::new(LdcType::Double(1f64))),
				InsnParser::DDIV => Insn::Divide(DivideInsn::new(PrimitiveType::Double)),
				InsnParser::DLOAD => {
					let index = rdr.read_u8()?;
					pc += 1;
					Insn::LocalLoad(LocalLoadInsn::new(OpType::Double, index as u16))
				},
				InsnParser::DLOAD_0 => Insn::LocalLoad(LocalLoadInsn::new(OpType::Double, 0)),
				InsnParser::DLOAD_1 => Insn::LocalLoad(LocalLoadInsn::new(OpType::Double, 1)),
				InsnParser::DLOAD_2 => Insn::LocalLoad(LocalLoadInsn::new(OpType::Double, 2)),
				InsnParser::DLOAD_3 => Insn::LocalLoad(LocalLoadInsn::new(OpType::Double, 3)),
				InsnParser::DMUL => Insn::Multiply(MultiplyInsn::new(PrimitiveType::Double)),
				InsnParser::DNEG => Insn::Negate(NegateInsn::new(PrimitiveType::Double)),
				InsnParser::DREM => Insn::Remainder(RemainderInsn::new(PrimitiveType::Double)),
				InsnParser::DRETURN => Insn::Return(ReturnInsn::new(ReturnType::Double)),
				InsnParser::DSTORE => {
					let index = rdr.read_u8()?;
					pc += 1;
					Insn::LocalStore(LocalStoreInsn::new(OpType::Double, index as u16))
				},
				InsnParser::DSTORE_0 => Insn::LocalStore(LocalStoreInsn::new(OpType::Double, 0)),
				InsnParser::DSTORE_1 => Insn::LocalStore(LocalStoreInsn::new(OpType::Double, 1)),
				InsnParser::DSTORE_2 => Insn::LocalStore(LocalStoreInsn::new(OpType::Double, 2)),
				InsnParser::DSTORE_3 => Insn::LocalStore(LocalStoreInsn::new(OpType::Double, 3)),
				InsnParser::DSUB => Insn::Subtract(SubtractInsn::new(PrimitiveType::Double)),
				InsnParser::DUP => Insn::Dup(DupInsn::new(1, 0)),
				InsnParser::DUP_X1 => Insn::Dup(DupInsn::new(1, 1)),
				InsnParser::DUP_X2 => Insn::Dup(DupInsn::new(1, 2)),
				InsnParser::DUP2 => Insn::Dup(DupInsn::new(2, 0)),
				InsnParser::DUP2_X1 => Insn::Dup(DupInsn::new(2, 1)),
				InsnParser::DUP2_X2 => Insn::Dup(DupInsn::new(2, 2)),
				InsnParser::F2D => Insn::Convert(ConvertInsn::new(PrimitiveType::Float, PrimitiveType::Double)),
				InsnParser::F2I => Insn::Convert(ConvertInsn::new(PrimitiveType::Float, PrimitiveType::Int)),
				InsnParser::F2L => Insn::Convert(ConvertInsn::new(PrimitiveType::Float, PrimitiveType::Long)),
				InsnParser::FADD => Insn::Add(AddInsn::new(PrimitiveType::Float)),
				InsnParser::FALOAD => Insn::ArrayLoad(ArrayLoadInsn::new(Type::Primitive(PrimitiveType::Float))),
				InsnParser::FASTORE => Insn::ArrayStore(ArrayStoreInsn::new(Type::Primitive(PrimitiveType::Float))),
				InsnParser::FCMPG => Insn::Compare(CompareInsn::new(PrimitiveType::Float, true)),
				InsnParser::FCMPL => Insn::Compare(CompareInsn::new(PrimitiveType::Float, false)),
				InsnParser::FCONST_0 => Insn::Ldc(LdcInsn::new(LdcType::Float(0f32))),
				InsnParser::FCONST_1 => Insn::Ldc(LdcInsn::new(LdcType::Float(1f32))),
				InsnParser::FCONST_2 => Insn::Ldc(LdcInsn::new(LdcType::Float(2f32))),
				InsnParser::FDIV => Insn::Divide(DivideInsn::new(PrimitiveType::Float)),
				InsnParser::FLOAD => {
					let index = rdr.read_u8()?;
					pc += 1;
					Insn::LocalLoad(LocalLoadInsn::new(OpType::Float, index as u16))
				},
				InsnParser::FLOAD_0 => Insn::LocalLoad(LocalLoadInsn::new(OpType::Float, 0)),
				InsnParser::FLOAD_1 => Insn::LocalLoad(LocalLoadInsn::new(OpType::Float, 1)),
				InsnParser::FLOAD_2 => Insn::LocalLoad(LocalLoadInsn::new(OpType::Float, 2)),
				InsnParser::FLOAD_3 => Insn::LocalLoad(LocalLoadInsn::new(OpType::Float, 3)),
				InsnParser::FMUL => Insn::Multiply(MultiplyInsn::new(PrimitiveType::Float)),
				InsnParser::FNEG => Insn::Negate(NegateInsn::new(PrimitiveType::Float)),
				InsnParser::FREM => Insn::Remainder(RemainderInsn::new(PrimitiveType::Float)),
				InsnParser::FRETURN => Insn::Return(ReturnInsn::new(ReturnType::Float)),
				InsnParser::FSTORE => {
					let index = rdr.read_u8()?;
					pc += 1;
					Insn::LocalStore(LocalStoreInsn::new(OpType::Float, index as u16))
				},
				InsnParser::FSTORE_0 => Insn::LocalStore(LocalStoreInsn::new(OpType::Float, 0)),
				InsnParser::FSTORE_1 => Insn::LocalStore(LocalStoreInsn::new(OpType::Float, 1)),
				InsnParser::FSTORE_2 => Insn::LocalStore(LocalStoreInsn::new(OpType::Float, 2)),
				InsnParser::FSTORE_3 => Insn::LocalStore(LocalStoreInsn::new(OpType::Float, 3)),
				InsnParser::FSUB => Insn::Subtract(SubtractInsn::new(PrimitiveType::Float)),
				InsnParser::GETFIELD => {
					let field_ref = constant_pool.fieldref(rdr.read_u16::<BigEndian>()?)?;
					pc += 2;
					let class = constant_pool.utf8(constant_pool.class(field_ref.class_index)?.name_index)?.str.clone();
					let name_type = constant_pool.nameandtype(field_ref.name_and_type_index)?;
					let name = constant_pool.utf8(name_type.name_index)?.str.clone();
					let descriptor = constant_pool.utf8(name_type.descriptor_index)?.str.clone();
					Insn::GetField(GetFieldInsn::new(true, class, name, descriptor))
				},
				InsnParser::GETSTATIC => {
					let field_ref = constant_pool.fieldref(rdr.read_u16::<BigEndian>()?)?;
					pc += 2;
					let class = constant_pool.utf8(constant_pool.class(field_ref.class_index)?.name_index)?.str.clone();
					let name_type = constant_pool.nameandtype(field_ref.name_and_type_index)?;
					let name = constant_pool.utf8(name_type.name_index)?.str.clone();
					let descriptor = constant_pool.utf8(name_type.descriptor_index)?.str.clone();
					Insn::GetField(GetFieldInsn::new(false, class, name, descriptor))
				},
				InsnParser::GOTO => {
					let to = (rdr.read_i16::<BigEndian>()? as i32 + this_pc as i32) as u32;
					pc += 2;
					required_labels += 1;
					Insn::Jump(JumpInsn::new(LabelInsn::new(to)))
				},
				InsnParser::GOTO_W => {
					let to = (rdr.read_i32::<BigEndian>()? + this_pc as i32) as u32;
					pc += 4;
					required_labels += 1;
					Insn::Jump(JumpInsn::new(LabelInsn::new(to)))
				},
				InsnParser::I2B => Insn::Convert(ConvertInsn::new(PrimitiveType::Int, PrimitiveType::Byte)),
				InsnParser::I2C => Insn::Convert(ConvertInsn::new(PrimitiveType::Int, PrimitiveType::Char)),
				InsnParser::I2D => Insn::Convert(ConvertInsn::new(PrimitiveType::Int, PrimitiveType::Double)),
				InsnParser::I2F => Insn::Convert(ConvertInsn::new(PrimitiveType::Int, PrimitiveType::Float)),
				InsnParser::I2L => Insn::Convert(ConvertInsn::new(PrimitiveType::Int, PrimitiveType::Long)),
				InsnParser::I2S => Insn::Convert(ConvertInsn::new(PrimitiveType::Int, PrimitiveType::Short)),
				InsnParser::IADD => Insn::Add(AddInsn::new(PrimitiveType::Int)),
				InsnParser::IALOAD => Insn::ArrayLoad(ArrayLoadInsn::new(Type::Primitive(PrimitiveType::Int))),
				InsnParser::IAND => Insn::And(AndInsn::new(PrimitiveType::Int)),
				InsnParser::IASTORE => Insn::ArrayStore(ArrayStoreInsn::new(Type::Primitive(PrimitiveType::Int))),
				InsnParser::ICONST_M1 => Insn::Ldc(LdcInsn::new(LdcType::Int(-1))),
				InsnParser::ICONST_0 => Insn::Ldc(LdcInsn::new(LdcType::Int(0))),
				InsnParser::ICONST_1 => Insn::Ldc(LdcInsn::new(LdcType::Int(1))),
				InsnParser::ICONST_2 => Insn::Ldc(LdcInsn::new(LdcType::Int(2))),
				InsnParser::ICONST_3 => Insn::Ldc(LdcInsn::new(LdcType::Int(3))),
				InsnParser::ICONST_4 => Insn::Ldc(LdcInsn::new(LdcType::Int(4))),
				InsnParser::ICONST_5 => Insn::Ldc(LdcInsn::new(LdcType::Int(5))),
				InsnParser::IDIV => Insn::Divide(DivideInsn::new(PrimitiveType::Int)),
				InsnParser::IF_ACMPEQ => {
					let to = (rdr.read_i16::<BigEndian>()? as i32 + this_pc as i32) as u32;
					pc += 2;
					required_labels += 1;
					Insn::ConditionalJump(ConditionalJumpInsn::new(JumpCondition::ReferencesEqual, LabelInsn::new(to as u32)))
				},
				InsnParser::IF_ACMPNE => {
					let to = (rdr.read_i16::<BigEndian>()? as i32 + this_pc as i32) as u32;
					pc += 2;
					required_labels += 1;
					Insn::ConditionalJump(ConditionalJumpInsn::new(JumpCondition::ReferencesNotEqual, LabelInsn::new(to as u32)))
				},
				InsnParser::IF_ICMPEQ => {
					let to = (rdr.read_i16::<BigEndian>()? as i32 + this_pc as i32) as u32;
					pc += 2;
					required_labels += 1;
					Insn::ConditionalJump(ConditionalJumpInsn::new(JumpCondition::IntsEq, LabelInsn::new(to as u32)))
				},
				InsnParser::IF_ICMPGE => {
					let to = (rdr.read_i16::<BigEndian>()? as i32 + this_pc as i32) as u32;
					pc += 2;
					required_labels += 1;
					Insn::ConditionalJump(ConditionalJumpInsn::new(JumpCondition::IntsGreaterThanOrEq, LabelInsn::new(to as u32)))
				},
				InsnParser::IF_ICMPGT => {
					let to = (rdr.read_i16::<BigEndian>()? as i32 + this_pc as i32) as u32;
					pc += 2;
					required_labels += 1;
					Insn::ConditionalJump(ConditionalJumpInsn::new(JumpCondition::IntsGreaterThan, LabelInsn::new(to as u32)))
				},
				InsnParser::IF_ICMPLE => {
					let to = (rdr.read_i16::<BigEndian>()? as i32 + this_pc as i32) as u32;
					pc += 2;
					required_labels += 1;
					Insn::ConditionalJump(ConditionalJumpInsn::new(JumpCondition::IntsLessThanOrEq, LabelInsn::new(to as u32)))
				},
				InsnParser::IF_ICMPLT => {
					let to = (rdr.read_i16::<BigEndian>()? as i32 + this_pc as i32) as u32;
					pc += 2;
					required_labels += 1;
					Insn::ConditionalJump(ConditionalJumpInsn::new(JumpCondition::IntsLessThan, LabelInsn::new(to as u32)))
				},
				InsnParser::IF_ICMPNE => {
					let to = (rdr.read_i16::<BigEndian>()? as i32 + this_pc as i32) as u32;
					pc += 2;
					required_labels += 1;
					Insn::ConditionalJump(ConditionalJumpInsn::new(JumpCondition::IntsNotEq, LabelInsn::new(to as u32)))
				},
				InsnParser::IFEQ => {
					let to = (rdr.read_i16::<BigEndian>()? as i32 + this_pc as i32) as u32;
					pc += 2;
					required_labels += 1;
					Insn::ConditionalJump(ConditionalJumpInsn::new(JumpCondition::IntEqZero, LabelInsn::new(to as u32)))
				},
				InsnParser::IFGE => {
					let to = (rdr.read_i16::<BigEndian>()? as i32 + this_pc as i32) as u32;
					pc += 2;
					required_labels += 1;
					Insn::ConditionalJump(ConditionalJumpInsn::new(JumpCondition::IntGreaterThanOrEqZero, LabelInsn::new(to as u32)))
				},
				InsnParser::IFGT => {
					let to = (rdr.read_i16::<BigEndian>()? as i32 + this_pc as i32) as u32;
					pc += 2;
					required_labels += 1;
					Insn::ConditionalJump(ConditionalJumpInsn::new(JumpCondition::IntGreaterThanZero, LabelInsn::new(to as u32)))
				},
				InsnParser::IFLE => {
					let to = (rdr.read_i16::<BigEndian>()? as i32 + this_pc as i32) as u32;
					pc += 2;
					required_labels += 1;
					Insn::ConditionalJump(ConditionalJumpInsn::new(JumpCondition::IntLessThanOrEqZero, LabelInsn::new(to as u32)))
				},
				InsnParser::IFLT => {
					let to = (rdr.read_i16::<BigEndian>()? as i32 + this_pc as i32) as u32;
					pc += 2;
					required_labels += 1;
					Insn::ConditionalJump(ConditionalJumpInsn::new(JumpCondition::IntLessThanZero, LabelInsn::new(to as u32)))
				},
				InsnParser::IFNE => {
					let to = (rdr.read_i16::<BigEndian>()? as i32 + this_pc as i32) as u32;
					pc += 2;
					required_labels += 1;
					Insn::ConditionalJump(ConditionalJumpInsn::new(JumpCondition::IntNotEqZero, LabelInsn::new(to as u32)))
				},
				InsnParser::IFNONNULL => {
					let to = (rdr.read_i16::<BigEndian>()? as i32 + this_pc as i32) as u32;
					pc += 2;
					required_labels += 1;
					Insn::ConditionalJump(ConditionalJumpInsn::new(JumpCondition::NotNull, LabelInsn::new(to as u32)))
				},
				InsnParser::IFNULL => {
					let to = (rdr.read_i16::<BigEndian>()? as i32 + this_pc as i32) as u32;
					pc += 2;
					required_labels += 1;
					Insn::ConditionalJump(ConditionalJumpInsn::new(JumpCondition::IsNull, LabelInsn::new(to as u32)))
				},
				InsnParser::IINC => {
					let index = rdr.read_u8()?;
					let amount = rdr.read_i8()?;
					pc += 2;
					Insn::IncrementInt(IncrementIntInsn::new(index as u16, amount as i16))
				},
				InsnParser::ILOAD => {
					let index = rdr.read_u8()?;
					pc += 1;
					Insn::LocalLoad(LocalLoadInsn::new(OpType::Int, index as u16))
				},
				InsnParser::ILOAD_0 => Insn::LocalLoad(LocalLoadInsn::new(OpType::Int, 0)),
				InsnParser::ILOAD_1 => Insn::LocalLoad(LocalLoadInsn::new(OpType::Int, 1)),
				InsnParser::ILOAD_2 => Insn::LocalLoad(LocalLoadInsn::new(OpType::Int, 2)),
				InsnParser::ILOAD_3 => Insn::LocalLoad(LocalLoadInsn::new(OpType::Int, 3)),
				InsnParser::IMPDEP1 => Insn::ImpDep1(ImpDep1Insn::new()),
				InsnParser::IMPDEP2 => Insn::ImpDep2(ImpDep2Insn::new()),
				InsnParser::IMUL => Insn::Multiply(MultiplyInsn::new(PrimitiveType::Int)),
				InsnParser::INEG => Insn::Negate(NegateInsn::new(PrimitiveType::Int)),
				InsnParser::INSTANCEOF => {
					let class = constant_pool.utf8(constant_pool.class(rdr.read_u16::<BigEndian>()?)?.name_index)?.str.clone();
					pc += 2;
					Insn::InstanceOf(InstanceOfInsn::new(class))
				},
				InsnParser::INVOKEDYNAMIC => {
					let dyn_info = constant_pool.invokedynamicinfo(rdr.read_u16::<BigEndian>()?)?;
					rdr.read_u16::<BigEndian>()?;
					pc += 4;
					// TODO: Resolve bootstrap methods
					
					let name_and_type = constant_pool.nameandtype(dyn_info.name_and_type_index)?;
					let name = constant_pool.utf8(name_and_type.name_index)?.str.clone();
					let descriptor = constant_pool.utf8(name_and_type.descriptor_index)?.str.clone();
					Insn::InvokeDynamic(InvokeDynamicInsn::new(name, descriptor, BootstrapMethodType::InvokeStatic, String::from("Unimplemented"), String::from("Unimplemented"), String::from("Unimplemented"), Vec::new()))
				},
				InsnParser::INVOKEINTERFACE => {
					let method = constant_pool.interfacemethodref(rdr.read_u16::<BigEndian>()?)?;
					let _count = rdr.read_u8()?; // serves 0 purpose? nice one jvm
					rdr.read_u8()?; // well at least it serves more purpose than this
					pc += 4;
					
					let name_and_type = constant_pool.nameandtype(method.name_and_type_index)?;
					let class = constant_pool.utf8(constant_pool.class(method.class_index)?.name_index)?.str.clone();
					let name = constant_pool.utf8(name_and_type.name_index)?.str.clone();
					let descriptor = constant_pool.utf8(name_and_type.descriptor_index)?.str.clone();
					Insn::Invoke(InvokeInsn::new(InvokeType::Instance, class, name, descriptor))
				}
				InsnParser::INVOKESPECIAL => {
					let method_index = rdr.read_u16::<BigEndian>()?;
					pc += 2;
					let (class, name, descriptor) = constant_pool.any_method(method_index)?;
					Insn::Invoke(InvokeInsn::new(InvokeType::Special, class, name, descriptor))
				},
				InsnParser::INVOKESTATIC => {
					let method_index = rdr.read_u16::<BigEndian>()?;
					pc += 2;
					let (class, name, descriptor) = constant_pool.any_method(method_index)?;
					Insn::Invoke(InvokeInsn::new(InvokeType::Static, class, name, descriptor))
				},
				InsnParser::INVOKEVIRTUAL => {
					let method_index = rdr.read_u16::<BigEndian>()?;
					pc += 2;
					let (class, name, descriptor) = constant_pool.any_method(method_index)?;
					Insn::Invoke(InvokeInsn::new(InvokeType::Instance, class, name, descriptor))
				},
				InsnParser::IOR => Insn::Or(OrInsn::new(PrimitiveType::Int)),
				InsnParser::IREM => Insn::Remainder(RemainderInsn::new(PrimitiveType::Int)),
				InsnParser::IRETURN => Insn::Return(ReturnInsn::new(ReturnType::Int)),
				InsnParser::ISHL => Insn::ShiftLeft(ShiftLeftInsn::new(OpType::Int)),
				InsnParser::ISHR => Insn::ShiftRight(ShiftRightInsn::new(OpType::Int)),
				InsnParser::ISTORE => {
					let index = rdr.read_u8()?;
					pc += 1;
					Insn::LocalStore(LocalStoreInsn::new(OpType::Int, index as u16))
				},
				InsnParser::ISTORE_0 => Insn::LocalStore(LocalStoreInsn::new(OpType::Int, 0)),
				InsnParser::ISTORE_1 => Insn::LocalStore(LocalStoreInsn::new(OpType::Int, 1)),
				InsnParser::ISTORE_2 => Insn::LocalStore(LocalStoreInsn::new(OpType::Int, 2)),
				InsnParser::ISTORE_3 => Insn::LocalStore(LocalStoreInsn::new(OpType::Int, 3)),
				InsnParser::ISUB => Insn::Subtract(SubtractInsn::new(PrimitiveType::Int)),
				InsnParser::IUSHR => Insn::LogicalShiftRight(LogicalShiftRightInsn::new(OpType::Int)),
				InsnParser::IXOR => Insn::Xor(XorInsn::new(PrimitiveType::Int)),
				//InsnParser::JSR =>
				//InsnParser::JSR_W =>
				InsnParser::L2D => Insn::Convert(ConvertInsn::new(PrimitiveType::Long, PrimitiveType::Double)),
				InsnParser::L2F => Insn::Convert(ConvertInsn::new(PrimitiveType::Long, PrimitiveType::Float)),
				InsnParser::L2I => Insn::Convert(ConvertInsn::new(PrimitiveType::Long, PrimitiveType::Int)),
				InsnParser::LADD => Insn::Add(AddInsn::new(PrimitiveType::Long)),
				InsnParser::LALOAD => Insn::ArrayLoad(ArrayLoadInsn::new(Type::Primitive(PrimitiveType::Long))),
				InsnParser::LASTORE => Insn::ArrayStore(ArrayStoreInsn::new(Type::Primitive(PrimitiveType::Long))),
				InsnParser::LCMP => Insn::Compare(CompareInsn::new(PrimitiveType::Long, false)),
				InsnParser::LCONST_0 => Insn::Ldc(LdcInsn::new(LdcType::Long(0))),
				InsnParser::LCONST_1 => Insn::Ldc(LdcInsn::new(LdcType::Long(1))),
				InsnParser::LDC => {
					let index = rdr.read_u8()? as u16;
					pc += 1;
					InsnParser::parse_ldc(index, constant_pool)?
				},
				InsnParser::LDC_W => {
					let index = rdr.read_u16::<BigEndian>()?;
					pc += 2;
					InsnParser::parse_ldc(index, constant_pool)?
				},
				InsnParser::LDC2_W => {
					let index = rdr.read_u16::<BigEndian>()?;
					pc += 2;
					InsnParser::parse_ldc(index, constant_pool)?
				},
				InsnParser::LDIV => Insn::Divide(DivideInsn::new(PrimitiveType::Long)),
				InsnParser::LLOAD => {
					let index = rdr.read_u8()?;
					pc += 1;
					Insn::LocalLoad(LocalLoadInsn::new(OpType::Double, index as u16))
				},
				InsnParser::LLOAD_0 => Insn::LocalLoad(LocalLoadInsn::new(OpType::Long, 0)),
				InsnParser::LLOAD_1 => Insn::LocalLoad(LocalLoadInsn::new(OpType::Long, 1)),
				InsnParser::LLOAD_2 => Insn::LocalLoad(LocalLoadInsn::new(OpType::Long, 2)),
				InsnParser::LLOAD_3 => Insn::LocalLoad(LocalLoadInsn::new(OpType::Long, 3)),
				InsnParser::LMUL => Insn::Multiply(MultiplyInsn::new(PrimitiveType::Long)),
				InsnParser::LNEG => Insn::Negate(NegateInsn::new(PrimitiveType::Long)),
				InsnParser::LOOKUPSWITCH => {
					let pad = 3 - (this_pc % 4);
					rdr.read_nbytes(pad as usize)?;
					
					let default = LabelInsn::new((rdr.read_i32::<BigEndian>()? + this_pc as i32) as u32);
					let npairs = rdr.read_i32::<BigEndian>()? as u32;
					
					let mut cases: HashMap<i32, LabelInsn> = HashMap::with_capacity(npairs as usize);
					for i in 0..npairs {
						let matc = rdr.read_i32::<BigEndian>()?;
						let jump = (rdr.read_i32::<BigEndian>()? + this_pc as i32) as u32;
						cases.insert(matc, LabelInsn::new(jump));
					}
					
					pc += pad + (2 * 4) + (npairs * 2 * 4);
					required_labels += npairs + 1;
					
					Insn::LookupSwitch(LookupSwitchInsn {
						default,
						cases
					})
				}
				InsnParser::LREM => Insn::Remainder(RemainderInsn::new(PrimitiveType::Long)),
				InsnParser::LRETURN => Insn::Return(ReturnInsn::new(ReturnType::Long)),
				InsnParser::LSHL => Insn::ShiftLeft(ShiftLeftInsn::new(OpType::Long)),
				InsnParser::LSHR => Insn::ShiftRight(ShiftRightInsn::new(OpType::Long)),
				InsnParser::LSTORE => {
					let index = rdr.read_u8()?;
					pc += 1;
					Insn::LocalStore(LocalStoreInsn::new(OpType::Long, index as u16))
				},
				InsnParser::LSTORE_0 => Insn::LocalStore(LocalStoreInsn::new(OpType::Long, 0)),
				InsnParser::LSTORE_1 => Insn::LocalStore(LocalStoreInsn::new(OpType::Long, 1)),
				InsnParser::LSTORE_2 => Insn::LocalStore(LocalStoreInsn::new(OpType::Long, 2)),
				InsnParser::LSTORE_3 => Insn::LocalStore(LocalStoreInsn::new(OpType::Long, 3)),
				InsnParser::LSUB => Insn::Subtract(SubtractInsn::new(PrimitiveType::Long)),
				InsnParser::LUSHR => Insn::LogicalShiftRight(LogicalShiftRightInsn::new(OpType::Long)),
				InsnParser::LXOR => Insn::Xor(XorInsn::new(PrimitiveType::Long)),
				InsnParser::MONITORENTER => Insn::MonitorEnter(MonitorEnterInsn::new()),
				InsnParser::MONITOREXIT => Insn::MonitorExit(MonitorExitInsn::new()),
				InsnParser::MULTIANEWARRAY => {
					let kind = constant_pool.utf8(constant_pool.class(rdr.read_u16::<BigEndian>()?)?.name_index)?.str.clone();
					pc += 2;
					let dimensions = rdr.read_u8()?;
					pc += 1;
					Insn::MultiNewArray(MultiNewArrayInsn::new(kind, dimensions))
				},
				InsnParser::NEW => {
					let kind = constant_pool.utf8(constant_pool.class(rdr.read_u16::<BigEndian>()?)?.name_index)?.str.clone();
					pc += 2;
					Insn::NewObject(NewObjectInsn::new(kind))
				},
				InsnParser::NEWARRAY => {
					let atype = rdr.read_u8()?;
					pc += 1;
					let kind = match atype {
						4 => PrimitiveType::Boolean,
						5 => PrimitiveType::Char,
						6 => PrimitiveType::Float,
						7 => PrimitiveType::Double,
						8 => PrimitiveType::Byte,
						9 => PrimitiveType::Short,
						10 => PrimitiveType::Int,
						11 => PrimitiveType::Long,
						_ => return Err(ParserError::other("Unknown Primitive Type"))
					};
					Insn::NewArray(NewArrayInsn::new(Type::Primitive(kind)))
				},
				InsnParser::NOP => Insn::Nop(NopInsn::new()),
				InsnParser::POP => Insn::Pop(PopInsn::new(1)),
				InsnParser::POP2 => Insn::Pop(PopInsn::new(2)),
				InsnParser::PUTFIELD => {
					let field_ref = constant_pool.fieldref(rdr.read_u16::<BigEndian>()?)?;
					pc += 2;
					let name_and_type = constant_pool.nameandtype(field_ref.name_and_type_index)?;
					let class = constant_pool.utf8(constant_pool.class(field_ref.class_index)?.name_index)?.str.clone();
					let name = constant_pool.utf8(name_and_type.name_index)?.str.clone();
					let desc = constant_pool.utf8(name_and_type.descriptor_index)?.str.clone();
					Insn::PutField(PutFieldInsn::new(true, class, name, desc))
				},
				InsnParser::PUTSTATIC => {
					let field_ref = constant_pool.fieldref(rdr.read_u16::<BigEndian>()?)?;
					pc += 2;
					let name_and_type = constant_pool.nameandtype(field_ref.name_and_type_index)?;
					let class = constant_pool.utf8(constant_pool.class(field_ref.class_index)?.name_index)?.str.clone();
					let name = constant_pool.utf8(name_and_type.name_index)?.str.clone();
					let desc = constant_pool.utf8(name_and_type.descriptor_index)?.str.clone();
					Insn::PutField(PutFieldInsn::new(false, class, name, desc))
				},
				//InsnParser::RET =>
				InsnParser::RETURN => Insn::Return(ReturnInsn::new(ReturnType::Void)),
				InsnParser::SALOAD => Insn::ArrayLoad(ArrayLoadInsn::new(Type::Primitive(PrimitiveType::Short))),
				InsnParser::SASTORE => Insn::ArrayStore(ArrayStoreInsn::new(Type::Primitive(PrimitiveType::Short))),
				InsnParser::SIPUSH => {
					let short = rdr.read_i16::<BigEndian>()?;
					pc += 2;
					Insn::Ldc(LdcInsn::new(LdcType::Int(short as i32)))
				},
				InsnParser::SWAP => Insn::Swap(SwapInsn::new()),
				InsnParser::TABLESWITCH => {
					let pad = 3 - (this_pc % 4);
					rdr.read_nbytes(pad as usize)?;
					
					let default = LabelInsn::new((rdr.read_i32::<BigEndian>()? + this_pc as i32) as u32);
					
					let low = rdr.read_i32::<BigEndian>()?;
					let high = rdr.read_i32::<BigEndian>()?;
					let num_cases = (high - low + 1) as u32;
					let mut cases: Vec<LabelInsn> = Vec::with_capacity(num_cases as usize);
					for i in 0..num_cases {
						let case = (rdr.read_i32::<BigEndian>()? + this_pc as i32) as u32;
						cases.push(LabelInsn::new(case));
					}
					
					pc += pad + ((3 + num_cases) * 4);
					required_labels += num_cases + 1;
					
					Insn::TableSwitch(TableSwitchInsn {
						default,
						low,
						cases
					})
				},
				InsnParser::WIDE => {
					let opcode = rdr.read_u8()?;
					pc += 1;
					match opcode {
						InsnParser::ILOAD => {
							let index = rdr.read_u16::<BigEndian>()?;
							pc += 2;
							Insn::LocalLoad(LocalLoadInsn::new(OpType::Int, index))
						},
						InsnParser::FLOAD => {
							let index = rdr.read_u16::<BigEndian>()?;
							pc += 2;
							Insn::LocalLoad(LocalLoadInsn::new(OpType::Float, index))
						},
						InsnParser::ALOAD => {
							let index = rdr.read_u16::<BigEndian>()?;
							pc += 2;
							Insn::LocalLoad(LocalLoadInsn::new(OpType::Reference, index))
						},
						InsnParser::LLOAD => {
							let index = rdr.read_u16::<BigEndian>()?;
							pc += 2;
							Insn::LocalLoad(LocalLoadInsn::new(OpType::Long, index))
						},
						InsnParser::DLOAD => {
							let index = rdr.read_u16::<BigEndian>()?;
							pc += 2;
							Insn::LocalLoad(LocalLoadInsn::new(OpType::Double, index))
						},
						InsnParser::ISTORE => {
							let index = rdr.read_u16::<BigEndian>()?;
							pc += 2;
							Insn::LocalStore(LocalStoreInsn::new(OpType::Int, index))
						},
						InsnParser::FSTORE => {
							let index = rdr.read_u16::<BigEndian>()?;
							pc += 2;
							Insn::LocalStore(LocalStoreInsn::new(OpType::Float, index))
						},
						InsnParser::LSTORE => {
							let index = rdr.read_u16::<BigEndian>()?;
							pc += 2;
							Insn::LocalStore(LocalStoreInsn::new(OpType::Long, index))
						},
						InsnParser::DSTORE => {
							let index = rdr.read_u16::<BigEndian>()?;
							pc += 2;
							Insn::LocalStore(LocalStoreInsn::new(OpType::Double, index))
						},
						InsnParser::RET => unimplemented!("Wide Ret instructions are not implemented"),
						_ => return Err(ParserError::invalid_insn(this_pc, format!("Invalid wide opcode {:x}", opcode)))
					}
				}
				_ => return Err(ParserError::unknown_insn(opcode))
			};
			//println!("{:#?}", insn);
			insns.push(insn);
			pc_index_map.insert(this_pc, index);
			
			index += 1;
		}
		
		let mut list = InsnList {
			insns: Vec::with_capacity(0),
			labels: 0
		};
		
		if required_labels > 0 {
			let mut insert: HashMap<usize, Vec<Insn>> = HashMap::with_capacity(required_labels as usize);
			// Remap labels to indexes
			for insn in insns.iter_mut() {
				match insn {
					Insn::Jump(x) => InsnParser::remap_label_nodes(&mut x.jump_to, &mut list, &pc_index_map, &mut insert)?,
					Insn::ConditionalJump(x) => InsnParser::remap_label_nodes(&mut x.jump_to, &mut list, &pc_index_map, &mut insert)?,
					Insn::TableSwitch(x) => {
						InsnParser::remap_label_nodes(&mut x.default, &mut list, &pc_index_map, &mut insert)?;
						for case in x.cases.iter_mut() {
							InsnParser::remap_label_nodes(case, &mut list, &pc_index_map, &mut insert)?
						}
					}
					Insn::LookupSwitch(x) => {
						InsnParser::remap_label_nodes(&mut x.default, &mut list, &pc_index_map, &mut insert)?;
						for (case, jump) in x.cases.iter_mut() {
							InsnParser::remap_label_nodes(jump, &mut list, &pc_index_map, &mut insert)?
						}
					}
					_ => {}
				}
			}
			insns.reserve_exact(insert.len());
			for (index, insert) in insert.iter_mut() {
				let index = *index;
				let mut empty = Vec::with_capacity(0);
				mem::swap(insert, &mut empty);
				for insn in empty.iter_mut() {
					let mut empty: Insn = unsafe { std::mem::zeroed() };
					mem::swap(insn, &mut empty);
					if index <= insns.len() {
						insns.insert(index, empty);
					} else {
						insns.push(empty);
					}
				}
			}
		}
		list.insns = insns;
		
		Ok(list)
	}
	
	fn remap_label_nodes(x: &mut LabelInsn, list: &mut InsnList, pc_index_map: &HashMap<u32, u32>, insert: &mut HashMap<usize, Vec<Insn>>) -> Result<()> {
		let jump_to = list.new_label();
		let mut insert_into = *match pc_index_map.get(&x.id) {
			Some(x) => x,
			_ => return Err(ParserError::out_of_bounds_jump(x.id as i32))
		};
		x.id = jump_to.id;
		
		for (i, insns) in insert.iter() {
			for _ in 0..insns.len() {
				if insert_into as usize > *i {
					insert_into += 1;
				}
			}
		}
		insert.entry(insert_into as usize)
			.or_insert(Vec::with_capacity(1))
			.push(Insn::Label(jump_to));
		Ok(())
	}
	
	fn parse_ldc(index: CPIndex, constant_pool: &ConstantPool) -> Result<Insn> {
		let constant = constant_pool.get(index)?;
		let ldc_type = match constant {
			ConstantType::String(x) => LdcType::String(constant_pool.utf8(x.string_index)?.str.clone()),
			ConstantType::Integer(x) => LdcType::Int(x.bytes),
			ConstantType::Float(x) => LdcType::Float(x.bytes),
			ConstantType::Double(x) => LdcType::Double(x.bytes),
			ConstantType::Long(x) => LdcType::Long(x.bytes),
			ConstantType::Class(x) => LdcType::Class(constant_pool.utf8(x.name_index)?.str.clone()),
			ConstantType::MethodType(x) => LdcType::MethodType(constant_pool.utf8(x.descriptor_index)?.str.clone()),
			ConstantType::MethodHandle(x) => return Err(ParserError::unimplemented("MethodHandle LDC")),
			ConstantType::Dynamic(x) => return Err(ParserError::unimplemented("Dynamic LDC")),
			x => return Err(ParserError::incomp_cp(
				"LDC Constant Type",
				constant,
				index as usize
			))
		};
		Ok(Insn::Ldc(LdcInsn::new(ldc_type)))
	}
}
