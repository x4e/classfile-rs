use crate::attributes::{Attribute, AttributeSource, Attributes};
use crate::constantpool::{ConstantPool, ConstantType, CPIndex, ConstantPoolWriter};
use crate::version::ClassVersion;
use crate::error::{Result, ParserError};
use crate::ast::*;
use crate::insnlist::InsnList;
use crate::utils::{ReadUtils, MapUtils};
use crate::types::{Type, parse_method_desc};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write, Cursor, Seek, SeekFrom};
use std::collections::HashMap;
use derive_more::Constructor;
use std::convert::TryFrom;

#[derive(Constructor, Clone, Debug, PartialEq)]
pub struct CodeAttribute {
	pub max_stack: u16,
	pub max_locals: u16,
	pub insns: InsnList,
	pub exceptions: Vec<ExceptionHandler>,
	pub attributes: Vec<Attribute>
}

impl CodeAttribute {
	pub fn empty() -> Self {
		CodeAttribute {
			max_stack: 0,
			max_locals: 0,
			insns: InsnList::with_capacity(0),
			exceptions: Vec::with_capacity(0),
			attributes: Vec::with_capacity(0)
		}
	}
	
	pub fn parse(version: &ClassVersion, constant_pool: &ConstantPool, buf: Vec<u8>) -> Result<Self> {
		let mut buf = Cursor::new(buf);
		
		let max_stack = buf.read_u16::<BigEndian>()?;
		let max_locals = buf.read_u16::<BigEndian>()?;
		
		let code_length = buf.read_u32::<BigEndian>()?;
		
		let code: Vec<u8> = buf.read_nbytes(code_length as usize)?;
		let mut code = Cursor::new(code);
		
		let mut pc_label_map: HashMap<u32, LabelInsn> = HashMap::new();
		InsnParser::find_insn_refs(&mut code, code_length, &mut pc_label_map)?;
		
		code.set_position(0);
		let code = InsnParser::parse_insns(constant_pool, &mut code, code_length, &mut pc_label_map)?;
		
		let num_exceptions = buf.read_u16::<BigEndian>()?;
		let mut exceptions: Vec<ExceptionHandler> = Vec::with_capacity(num_exceptions as usize);
		for _ in 0..num_exceptions {
			exceptions.push(ExceptionHandler::parse(constant_pool, &mut buf)?);
		}
		
		let attributes = Attributes::parse(&mut buf, AttributeSource::Code, version, constant_pool)?;
		
		Ok(CodeAttribute {
			max_stack,
			max_locals,
			insns: code,
			exceptions,
			attributes
		})
	}
	
	pub fn write<T: Write>(&self, wtr: &mut T, constant_pool: &mut ConstantPoolWriter) -> Result<()> {
		wtr.write_u16::<BigEndian>(self.max_stack)?;
		wtr.write_u16::<BigEndian>(self.max_locals)?;
		let code_bytes = InsnParser::write_insns(self, constant_pool)?;
		wtr.write_u32::<BigEndian>(code_bytes.len() as u32)?;
		wtr.write_all(code_bytes.as_slice())?;
		wtr.write_u16::<BigEndian>(self.exceptions.len() as u16)?;
		for excep in self.exceptions.iter() {
			excep.write(wtr, constant_pool)?;
		}
		Attributes::write(wtr, &self.attributes, constant_pool)?;
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
	// TODO: exception handlers should use labels
	pub fn parse<T: Read>(constant_pool: &ConstantPool, buf: &mut T) -> Result<Self> {
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
	
	pub fn write<T: Write>(&self, wtr: &mut T, constant_pool: &mut ConstantPoolWriter) -> Result<()> {
		wtr.write_u16::<BigEndian>(self.start_pc)?;
		wtr.write_u16::<BigEndian>(self.end_pc)?;
		wtr.write_u16::<BigEndian>(self.handler_pc)?;
		let catch_type = match self.catch_type.clone() {
			Some(x) => constant_pool.class_utf8(x),
			None => 0
		};
		wtr.write_u16::<BigEndian>(catch_type)?;
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
	
	/// Iterate all instructions and collect any pcs that are referenced - i.e. need to have relevant Labels
	fn find_insn_refs<T: Read + Seek>(rdr: &mut T, length: u32, pc_label_map: &mut HashMap<u32, LabelInsn>) -> Result<()> {
		let mut pc: u32 = 0;
		while pc < length {
			let this_pc = pc;
			let opcode = rdr.read_u8()?;
			pc += 1;
			
			match opcode {
				InsnParser::GOTO => {
					let to = (rdr.read_i16::<BigEndian>()? as i32 + this_pc as i32) as u32;
					pc_label_map.insert_if_not_present(to, LabelInsn::new(pc_label_map.len() as u32));
					pc += 2;
				}
				InsnParser::GOTO_W => {
					let to = (rdr.read_i32::<BigEndian>()? + this_pc as i32) as u32;
					pc_label_map.insert_if_not_present(to, LabelInsn::new(pc_label_map.len() as u32));
					pc += 4;
				}
				InsnParser::IF_ACMPEQ => {
					let to = (rdr.read_i16::<BigEndian>()? as i32 + this_pc as i32) as u32;
					pc_label_map.insert_if_not_present(to, LabelInsn::new(pc_label_map.len() as u32));
					pc += 2;
				}
				InsnParser::IF_ACMPNE => {
					let to = (rdr.read_i16::<BigEndian>()? as i32 + this_pc as i32) as u32;
					pc_label_map.insert_if_not_present(to, LabelInsn::new(pc_label_map.len() as u32));
					pc += 2;
				}
				InsnParser::IF_ICMPEQ => {
					let to = (rdr.read_i16::<BigEndian>()? as i32 + this_pc as i32) as u32;
					pc_label_map.insert_if_not_present(to, LabelInsn::new(pc_label_map.len() as u32));
					pc += 2;
				}
				InsnParser::IF_ICMPGE => {
					let to = (rdr.read_i16::<BigEndian>()? as i32 + this_pc as i32) as u32;
					pc_label_map.insert_if_not_present(to, LabelInsn::new(pc_label_map.len() as u32));
					pc += 2;
				}
				InsnParser::IF_ICMPGT => {
					let to = (rdr.read_i16::<BigEndian>()? as i32 + this_pc as i32) as u32;
					pc_label_map.insert_if_not_present(to, LabelInsn::new(pc_label_map.len() as u32));
					pc += 2;
				}
				InsnParser::IF_ICMPLE => {
					let to = (rdr.read_i16::<BigEndian>()? as i32 + this_pc as i32) as u32;
					pc_label_map.insert_if_not_present(to, LabelInsn::new(pc_label_map.len() as u32));
					pc += 2;
				}
				InsnParser::IF_ICMPLT => {
					let to = (rdr.read_i16::<BigEndian>()? as i32 + this_pc as i32) as u32;
					pc_label_map.insert_if_not_present(to, LabelInsn::new(pc_label_map.len() as u32));
					pc += 2;
				}
				InsnParser::IF_ICMPNE => {
					let to = (rdr.read_i16::<BigEndian>()? as i32 + this_pc as i32) as u32;
					pc_label_map.insert_if_not_present(to, LabelInsn::new(pc_label_map.len() as u32));
					pc += 2;
				}
				InsnParser::IFEQ => {
					let to = (rdr.read_i16::<BigEndian>()? as i32 + this_pc as i32) as u32;
					pc_label_map.insert_if_not_present(to, LabelInsn::new(pc_label_map.len() as u32));
					pc += 2;
				}
				InsnParser::IFGE => {
					let to = (rdr.read_i16::<BigEndian>()? as i32 + this_pc as i32) as u32;
					pc_label_map.insert_if_not_present(to, LabelInsn::new(pc_label_map.len() as u32));
					pc += 2;
				}
				InsnParser::IFGT => {
					let to = (rdr.read_i16::<BigEndian>()? as i32 + this_pc as i32) as u32;
					pc_label_map.insert_if_not_present(to, LabelInsn::new(pc_label_map.len() as u32));
					pc += 2;
				}
				InsnParser::IFLE => {
					let to = (rdr.read_i16::<BigEndian>()? as i32 + this_pc as i32) as u32;
					pc_label_map.insert_if_not_present(to, LabelInsn::new(pc_label_map.len() as u32));
					pc += 2;
				}
				InsnParser::IFLT => {
					let to = (rdr.read_i16::<BigEndian>()? as i32 + this_pc as i32) as u32;
					pc_label_map.insert_if_not_present(to, LabelInsn::new(pc_label_map.len() as u32));
					pc += 2;
				}
				InsnParser::IFNE => {
					let to = (rdr.read_i16::<BigEndian>()? as i32 + this_pc as i32) as u32;
					pc_label_map.insert_if_not_present(to, LabelInsn::new(pc_label_map.len() as u32));
					pc += 2;
				}
				InsnParser::IFNONNULL => {
					let to = (rdr.read_i16::<BigEndian>()? as i32 + this_pc as i32) as u32;
					pc_label_map.insert_if_not_present(to, LabelInsn::new(pc_label_map.len() as u32));
					pc += 2;
				}
				InsnParser::IFNULL => {
					let to = (rdr.read_i16::<BigEndian>()? as i32 + this_pc as i32) as u32;
					pc_label_map.insert_if_not_present(to, LabelInsn::new(pc_label_map.len() as u32));
					pc += 2;
				}
				InsnParser::LOOKUPSWITCH => {
					let pad = 3 - (this_pc % 4);
					rdr.seek(SeekFrom::Current(pad as i64))?;
					
					let default = (rdr.read_i32::<BigEndian>()? + this_pc as i32) as u32;
					pc_label_map.insert_if_not_present(default, LabelInsn::new(pc_label_map.len() as u32));
					let npairs = rdr.read_i32::<BigEndian>()? as u32;
					
					for i in 0..npairs {
						let matc = rdr.read_i32::<BigEndian>()?;
						let jump = (rdr.read_i32::<BigEndian>()? + this_pc as i32) as u32;
						pc_label_map.insert_if_not_present(jump, LabelInsn::new(pc_label_map.len() as u32));
					}
					
					pc += pad + (2 * 4) + (npairs * 2 * 4);
				}
				InsnParser::TABLESWITCH => {
					let pad = 3 - (this_pc % 4);
					rdr.seek(SeekFrom::Current(pad as i64))?;
					
					let default = (rdr.read_i32::<BigEndian>()? + this_pc as i32) as u32;
					pc_label_map.insert_if_not_present(default, LabelInsn::new(pc_label_map.len() as u32));
					
					let low = rdr.read_i32::<BigEndian>()?;
					let high = rdr.read_i32::<BigEndian>()?;
					let num_cases = (high - low + 1) as u32;
					for i in 0..num_cases {
						let case = (rdr.read_i32::<BigEndian>()? + this_pc as i32) as u32;
						pc_label_map.insert_if_not_present(case, LabelInsn::new(pc_label_map.len() as u32));
					}
					
					pc += pad + ((3 + num_cases) * 4);
				},
				InsnParser::AALOAD | InsnParser::AASTORE | InsnParser::ACONST_NULL |
				InsnParser::ALOAD_0 | InsnParser::ALOAD_1 | InsnParser::ALOAD_2 |
				InsnParser::ALOAD_3 | InsnParser::ARETURN | InsnParser::ARRAYLENGTH |
				InsnParser::ASTORE_0 | InsnParser::ASTORE_2 | InsnParser::ASTORE_3 |
				InsnParser::ATHROW | InsnParser::BALOAD | InsnParser::BASTORE |
				InsnParser::BREAKPOINT | InsnParser::CALOAD | InsnParser::CASTORE |
				InsnParser::D2F | InsnParser::D2I | InsnParser::D2L | InsnParser::DADD |
				InsnParser::DALOAD | InsnParser::DASTORE | InsnParser::DCMPG | InsnParser::DCMPL |
				InsnParser::DCONST_0 | InsnParser::DCONST_1 | InsnParser::DDIV |
				InsnParser::DLOAD_0 | InsnParser::DLOAD_1 | InsnParser::DLOAD_2 |
				InsnParser::DLOAD_3 | InsnParser::DMUL | InsnParser::DNEG | InsnParser::DREM |
				InsnParser::DRETURN | InsnParser::DSTORE_0 | InsnParser::DSTORE_1 |
				InsnParser::DSTORE_2 | InsnParser::DSTORE_3 | InsnParser::DSUB | InsnParser::DUP |
				InsnParser::DUP_X1 | InsnParser::DUP_X2 | InsnParser::DUP2 | InsnParser::DUP2_X1 |
				InsnParser::DUP2_X2 | InsnParser::F2D | InsnParser::F2I | InsnParser::F2L |
				InsnParser::FADD | InsnParser::FALOAD | InsnParser::FASTORE | InsnParser::FCMPG |
				InsnParser::FCMPL | InsnParser::FCONST_0 | InsnParser::FCONST_1 |
				InsnParser::FCONST_2 | InsnParser::FDIV | InsnParser::FLOAD_0 |
				InsnParser::FLOAD_1 | InsnParser::FLOAD_2 | InsnParser::FLOAD_3 | InsnParser::FMUL |
				InsnParser::FNEG | InsnParser::FREM | InsnParser::FRETURN | InsnParser::FSTORE_0 |
				InsnParser::FSTORE_1 | InsnParser::FSTORE_2 | InsnParser::FSTORE_3 |
				InsnParser::FSUB | InsnParser::I2B | InsnParser::I2C | InsnParser::I2D |
				InsnParser::I2F | InsnParser::I2L | InsnParser::I2S | InsnParser::IADD |
				InsnParser::IALOAD | InsnParser::IAND | InsnParser::IASTORE |
				InsnParser::ICONST_M1 | InsnParser::ICONST_0 | InsnParser::ICONST_1 |
				InsnParser::ICONST_2 | InsnParser::ICONST_3 | InsnParser::ICONST_4 |
				InsnParser::ICONST_5 | InsnParser::IDIV | InsnParser::ILOAD_0 |
				InsnParser::ILOAD_1 | InsnParser::ILOAD_2 | InsnParser::ILOAD_3 |
				InsnParser::IMPDEP1 | InsnParser::IMPDEP2 | InsnParser::IMUL | InsnParser::INEG |
				InsnParser::IOR | InsnParser::IREM | InsnParser::IRETURN | InsnParser::ISHL |
				InsnParser::ISHR | InsnParser::ISTORE_0 | InsnParser::ISTORE_1 |
				InsnParser::ISTORE_2 | InsnParser::ISTORE_3 | InsnParser::ISUB | InsnParser::IUSHR |
				InsnParser::IXOR | InsnParser::L2D | InsnParser::L2F | InsnParser::L2I |
				InsnParser::LADD | InsnParser::LALOAD | InsnParser::LAND | InsnParser::LASTORE |
				InsnParser::LCMP | InsnParser::LCONST_0 | InsnParser::LCONST_1 | InsnParser::LDIV |
				InsnParser::LLOAD_0 | InsnParser::LLOAD_1 | InsnParser::LLOAD_2 |
				InsnParser::LLOAD_3 | InsnParser::LMUL | InsnParser::LNEG | InsnParser::LOR |
				InsnParser::LREM | InsnParser::LRETURN | InsnParser::LSHL | InsnParser::LSHR |
				InsnParser::LSTORE_0 | InsnParser::LSTORE_1 | InsnParser::LSTORE_2 |
				InsnParser::LSTORE_3 | InsnParser::LSUB | InsnParser::LUSHR | InsnParser::LXOR |
				InsnParser::MONITORENTER | InsnParser::MONITOREXIT | InsnParser::NOP |
				InsnParser::POP | InsnParser::POP2 | InsnParser::RETURN | InsnParser::SALOAD |
				InsnParser::SASTORE | InsnParser::SWAP => {},
				InsnParser::ALOAD | InsnParser::ASTORE | InsnParser::BIPUSH | InsnParser::DLOAD |
				InsnParser::DSTORE | InsnParser::FLOAD | InsnParser::FSTORE | InsnParser::ILOAD |
				InsnParser::ISTORE | InsnParser::LDC | InsnParser::LLOAD | InsnParser::LSTORE |
				InsnParser::NEWARRAY => {
					pc += 1;
					rdr.seek(SeekFrom::Current(1))?;
				}
				InsnParser::ANEWARRAY | InsnParser::CHECKCAST | InsnParser::GETFIELD |
				InsnParser::GETSTATIC | InsnParser::IINC | InsnParser::INSTANCEOF |
				InsnParser::INVOKESPECIAL | InsnParser::INVOKESTATIC | InsnParser::INVOKEVIRTUAL |
				InsnParser::LDC_W | InsnParser::LDC2_W | InsnParser::NEW | InsnParser::PUTFIELD |
				InsnParser::PUTSTATIC | InsnParser::SIPUSH => {
					pc += 2;
					rdr.seek(SeekFrom::Current(2))?;
				}
				InsnParser::MULTIANEWARRAY => {
					pc += 3;
					rdr.seek(SeekFrom::Current(3))?;
				}
				InsnParser::INVOKEDYNAMIC | InsnParser::INVOKEINTERFACE => {
					pc += 4;
					rdr.seek(SeekFrom::Current(4))?;
				}
				InsnParser::WIDE => match rdr.read_u8()? {
					InsnParser::ILOAD | InsnParser::FLOAD | InsnParser::ALOAD | InsnParser::LLOAD | InsnParser::DLOAD | InsnParser::ISTORE | InsnParser::FSTORE | InsnParser::LSTORE | InsnParser::DSTORE => {
						pc += 3;
						rdr.seek(SeekFrom::Current(3))?;
					}
					InsnParser::IINC => {
						pc += 5;
						rdr.seek(SeekFrom::Current(5))?;
					}
					_ => return Err(ParserError::invalid_insn(this_pc, format!("Invalid wide opcode {:x}", opcode)))
				},
				_ => return Err(ParserError::unknown_insn(opcode))
			}
		}
		Ok(())
	}
	
	fn parse_insns<T: Read>(constant_pool: &ConstantPool, mut rdr: T, length: u32, pc_label_map: &mut HashMap<u32, LabelInsn>) -> Result<InsnList> {
		let num_insns_estimate = length as usize / 3; // estimate an average 3 bytes per insn
		let mut insns: Vec<Insn> = Vec::with_capacity(num_insns_estimate);
		
		let mut pc: u32 = 0;
		let mut index: u32 = 0;
		while pc < length {
			let this_pc = pc;
			let opcode = rdr.read_u8()?;
			pc += 1;
			
			// does this pc need an associated label?
			if let Some(lbl) = pc_label_map.get(&this_pc) {
				insns.push(Insn::Label(*lbl));
			}
			
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
				InsnParser::ASTORE_0 => Insn::LocalStore(LocalStoreInsn::new(OpType::Reference, 0)),
				InsnParser::ASTORE_1 => Insn::LocalStore(LocalStoreInsn::new(OpType::Reference, 1)),
				InsnParser::ASTORE_2 => Insn::LocalStore(LocalStoreInsn::new(OpType::Reference, 2)),
				InsnParser::ASTORE_3 => Insn::LocalStore(LocalStoreInsn::new(OpType::Reference, 3)),
				InsnParser::ATHROW => Insn::Throw(ThrowInsn::new()),
				// BALOAD is both byte and boolean (they are same size on hotspot) we will assume byte
				InsnParser::BALOAD => Insn::ArrayLoad(ArrayLoadInsn::new(Type::Byte)),
				InsnParser::BASTORE => Insn::ArrayStore(ArrayStoreInsn::new(Type::Byte)),
				InsnParser::BIPUSH => {
					let byte = rdr.read_i8()?;
					pc += 1;
					Insn::Ldc(LdcInsn::new(LdcType::Int(byte as i32)))
				},
				InsnParser::BREAKPOINT => Insn::BreakPoint(BreakPointInsn::new()),
				InsnParser::CALOAD => Insn::ArrayLoad(ArrayLoadInsn::new(Type::Char)),
				InsnParser::CASTORE => Insn::ArrayStore(ArrayStoreInsn::new(Type::Char)),
				InsnParser::CHECKCAST => {
					let kind = constant_pool.utf8(constant_pool.class(rdr.read_u16::<BigEndian>()?)?.name_index)?.str.clone();
					pc += 2;
					Insn::CheckCast(CheckCastInsn::new(kind))
				},
				InsnParser::D2F => Insn::Convert(ConvertInsn::new(PrimitiveType::Double, PrimitiveType::Float)),
				InsnParser::D2I => Insn::Convert(ConvertInsn::new(PrimitiveType::Double, PrimitiveType::Int)),
				InsnParser::D2L => Insn::Convert(ConvertInsn::new(PrimitiveType::Double, PrimitiveType::Long)),
				InsnParser::DADD => Insn::Add(AddInsn::new(PrimitiveType::Double)),
				InsnParser::DALOAD => Insn::ArrayLoad(ArrayLoadInsn::new(Type::Double)),
				InsnParser::DASTORE => Insn::ArrayStore(ArrayStoreInsn::new(Type::Double)),
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
				InsnParser::FALOAD => Insn::ArrayLoad(ArrayLoadInsn::new(Type::Float)),
				InsnParser::FASTORE => Insn::ArrayStore(ArrayStoreInsn::new(Type::Float)),
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
					Insn::Jump(JumpInsn::new(*pc_label_map.get(&to).ok_or_else(ParserError::unmapped_label)?))
				},
				InsnParser::GOTO_W => {
					let to = (rdr.read_i32::<BigEndian>()? + this_pc as i32) as u32;
					pc += 4;
					Insn::Jump(JumpInsn::new(*pc_label_map.get(&to).ok_or_else(ParserError::unmapped_label)?))
				},
				InsnParser::I2B => Insn::Convert(ConvertInsn::new(PrimitiveType::Int, PrimitiveType::Byte)),
				InsnParser::I2C => Insn::Convert(ConvertInsn::new(PrimitiveType::Int, PrimitiveType::Char)),
				InsnParser::I2D => Insn::Convert(ConvertInsn::new(PrimitiveType::Int, PrimitiveType::Double)),
				InsnParser::I2F => Insn::Convert(ConvertInsn::new(PrimitiveType::Int, PrimitiveType::Float)),
				InsnParser::I2L => Insn::Convert(ConvertInsn::new(PrimitiveType::Int, PrimitiveType::Long)),
				InsnParser::I2S => Insn::Convert(ConvertInsn::new(PrimitiveType::Int, PrimitiveType::Short)),
				InsnParser::IADD => Insn::Add(AddInsn::new(PrimitiveType::Int)),
				InsnParser::IALOAD => Insn::ArrayLoad(ArrayLoadInsn::new(Type::Int)),
				InsnParser::IAND => Insn::And(AndInsn::new(IntegerType::Int)),
				InsnParser::IASTORE => Insn::ArrayStore(ArrayStoreInsn::new(Type::Int)),
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
					Insn::ConditionalJump(ConditionalJumpInsn::new(JumpCondition::ReferencesEqual, *pc_label_map.get(&to).ok_or_else(ParserError::unmapped_label)?))
				},
				InsnParser::IF_ACMPNE => {
					let to = (rdr.read_i16::<BigEndian>()? as i32 + this_pc as i32) as u32;
					pc += 2;
					Insn::ConditionalJump(ConditionalJumpInsn::new(JumpCondition::ReferencesNotEqual, *pc_label_map.get(&to).ok_or_else(ParserError::unmapped_label)?))
				},
				InsnParser::IF_ICMPEQ => {
					let to = (rdr.read_i16::<BigEndian>()? as i32 + this_pc as i32) as u32;
					pc += 2;
					Insn::ConditionalJump(ConditionalJumpInsn::new(JumpCondition::IntsEq, *pc_label_map.get(&to).ok_or_else(ParserError::unmapped_label)?))
				},
				InsnParser::IF_ICMPGE => {
					let to = (rdr.read_i16::<BigEndian>()? as i32 + this_pc as i32) as u32;
					pc += 2;
					Insn::ConditionalJump(ConditionalJumpInsn::new(JumpCondition::IntsGreaterThanOrEq, *pc_label_map.get(&to).ok_or_else(ParserError::unmapped_label)?))
				},
				InsnParser::IF_ICMPGT => {
					let to = (rdr.read_i16::<BigEndian>()? as i32 + this_pc as i32) as u32;
					pc += 2;
					Insn::ConditionalJump(ConditionalJumpInsn::new(JumpCondition::IntsGreaterThan, *pc_label_map.get(&to).ok_or_else(ParserError::unmapped_label)?))
				},
				InsnParser::IF_ICMPLE => {
					let to = (rdr.read_i16::<BigEndian>()? as i32 + this_pc as i32) as u32;
					pc += 2;
					Insn::ConditionalJump(ConditionalJumpInsn::new(JumpCondition::IntsLessThanOrEq, *pc_label_map.get(&to).ok_or_else(ParserError::unmapped_label)?))
				},
				InsnParser::IF_ICMPLT => {
					let to = (rdr.read_i16::<BigEndian>()? as i32 + this_pc as i32) as u32;
					pc += 2;
					Insn::ConditionalJump(ConditionalJumpInsn::new(JumpCondition::IntsLessThan, *pc_label_map.get(&to).ok_or_else(ParserError::unmapped_label)?))
				},
				InsnParser::IF_ICMPNE => {
					let to = (rdr.read_i16::<BigEndian>()? as i32 + this_pc as i32) as u32;
					pc += 2;
					Insn::ConditionalJump(ConditionalJumpInsn::new(JumpCondition::IntsNotEq, *pc_label_map.get(&to).ok_or_else(ParserError::unmapped_label)?))
				},
				InsnParser::IFEQ => {
					let to = (rdr.read_i16::<BigEndian>()? as i32 + this_pc as i32) as u32;
					pc += 2;
					Insn::ConditionalJump(ConditionalJumpInsn::new(JumpCondition::IntEqZero, *pc_label_map.get(&to).ok_or_else(ParserError::unmapped_label)?))
				},
				InsnParser::IFGE => {
					let to = (rdr.read_i16::<BigEndian>()? as i32 + this_pc as i32) as u32;
					pc += 2;
					Insn::ConditionalJump(ConditionalJumpInsn::new(JumpCondition::IntGreaterThanOrEqZero, *pc_label_map.get(&to).ok_or_else(ParserError::unmapped_label)?))
				},
				InsnParser::IFGT => {
					let to = (rdr.read_i16::<BigEndian>()? as i32 + this_pc as i32) as u32;
					pc += 2;
					Insn::ConditionalJump(ConditionalJumpInsn::new(JumpCondition::IntGreaterThanZero, *pc_label_map.get(&to).ok_or_else(ParserError::unmapped_label)?))
				},
				InsnParser::IFLE => {
					let to = (rdr.read_i16::<BigEndian>()? as i32 + this_pc as i32) as u32;
					pc += 2;
					Insn::ConditionalJump(ConditionalJumpInsn::new(JumpCondition::IntLessThanOrEqZero, *pc_label_map.get(&to).ok_or_else(ParserError::unmapped_label)?))
				},
				InsnParser::IFLT => {
					let to = (rdr.read_i16::<BigEndian>()? as i32 + this_pc as i32) as u32;
					pc += 2;
					Insn::ConditionalJump(ConditionalJumpInsn::new(JumpCondition::IntLessThanZero, *pc_label_map.get(&to).ok_or_else(ParserError::unmapped_label)?))
				},
				InsnParser::IFNE => {
					let to = (rdr.read_i16::<BigEndian>()? as i32 + this_pc as i32) as u32;
					pc += 2;
					Insn::ConditionalJump(ConditionalJumpInsn::new(JumpCondition::IntNotEqZero, *pc_label_map.get(&to).ok_or_else(ParserError::unmapped_label)?))
				},
				InsnParser::IFNONNULL => {
					let to = (rdr.read_i16::<BigEndian>()? as i32 + this_pc as i32) as u32;
					pc += 2;
					Insn::ConditionalJump(ConditionalJumpInsn::new(JumpCondition::NotNull, *pc_label_map.get(&to).ok_or_else(ParserError::unmapped_label)?))
				},
				InsnParser::IFNULL => {
					let to = (rdr.read_i16::<BigEndian>()? as i32 + this_pc as i32) as u32;
					pc += 2;
					Insn::ConditionalJump(ConditionalJumpInsn::new(JumpCondition::IsNull, *pc_label_map.get(&to).ok_or_else(ParserError::unmapped_label)?))
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
					Insn::Invoke(InvokeInsn::new(InvokeType::Instance, class, name, descriptor, true))
				}
				InsnParser::INVOKESPECIAL => {
					let method_index = rdr.read_u16::<BigEndian>()?;
					pc += 2;
					
					let (method, interface_method) = constant_pool.any_method(method_index)?;
					let name_and_type = constant_pool.nameandtype(method.name_and_type_index)?;
					let class = constant_pool.utf8(constant_pool.class(method.class_index)?.name_index)?.str.clone();
					let name = constant_pool.utf8(name_and_type.name_index)?.str.clone();
					let descriptor = constant_pool.utf8(name_and_type.descriptor_index)?.str.clone();
					
					Insn::Invoke(InvokeInsn::new(InvokeType::Special, class, name, descriptor, interface_method))
				},
				InsnParser::INVOKESTATIC => {
					let method_index = rdr.read_u16::<BigEndian>()?;
					pc += 2;
					
					let (method, interface_method) = constant_pool.any_method(method_index)?;
					let name_and_type = constant_pool.nameandtype(method.name_and_type_index)?;
					let class = constant_pool.utf8(constant_pool.class(method.class_index)?.name_index)?.str.clone();
					let name = constant_pool.utf8(name_and_type.name_index)?.str.clone();
					let descriptor = constant_pool.utf8(name_and_type.descriptor_index)?.str.clone();
					
					Insn::Invoke(InvokeInsn::new(InvokeType::Static, class, name, descriptor, interface_method))
				},
				InsnParser::INVOKEVIRTUAL => {
					let method_index = rdr.read_u16::<BigEndian>()?;
					pc += 2;
					
					let (method, interface_method) = constant_pool.any_method(method_index)?;
					let name_and_type = constant_pool.nameandtype(method.name_and_type_index)?;
					let class = constant_pool.utf8(constant_pool.class(method.class_index)?.name_index)?.str.clone();
					let name = constant_pool.utf8(name_and_type.name_index)?.str.clone();
					let descriptor = constant_pool.utf8(name_and_type.descriptor_index)?.str.clone();
					
					Insn::Invoke(InvokeInsn::new(InvokeType::Instance, class, name, descriptor, interface_method))
				},
				InsnParser::IOR => Insn::Or(OrInsn::new(IntegerType::Int)),
				InsnParser::IREM => Insn::Remainder(RemainderInsn::new(PrimitiveType::Int)),
				InsnParser::IRETURN => Insn::Return(ReturnInsn::new(ReturnType::Int)),
				InsnParser::ISHL => Insn::ShiftLeft(ShiftLeftInsn::new(IntegerType::Int)),
				InsnParser::ISHR => Insn::ShiftRight(ShiftRightInsn::new(IntegerType::Int)),
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
				InsnParser::IUSHR => Insn::LogicalShiftRight(LogicalShiftRightInsn::new(IntegerType::Int)),
				InsnParser::IXOR => Insn::Xor(XorInsn::new(IntegerType::Int)),
				//InsnParser::JSR =>
				//InsnParser::JSR_W =>
				InsnParser::L2D => Insn::Convert(ConvertInsn::new(PrimitiveType::Long, PrimitiveType::Double)),
				InsnParser::L2F => Insn::Convert(ConvertInsn::new(PrimitiveType::Long, PrimitiveType::Float)),
				InsnParser::L2I => Insn::Convert(ConvertInsn::new(PrimitiveType::Long, PrimitiveType::Int)),
				InsnParser::LADD => Insn::Add(AddInsn::new(PrimitiveType::Long)),
				InsnParser::LALOAD => Insn::ArrayLoad(ArrayLoadInsn::new(Type::Long)),
				InsnParser::LAND => Insn::And(AndInsn::new(IntegerType::Long)),
				InsnParser::LASTORE => Insn::ArrayStore(ArrayStoreInsn::new(Type::Long)),
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
					
					let default = (rdr.read_i32::<BigEndian>()? + this_pc as i32) as u32;
					let npairs = rdr.read_i32::<BigEndian>()? as u32;
					
					let mut insn = LookupSwitchInsn::new(*pc_label_map.get(&default).ok_or_else(ParserError::unmapped_label)?);
					
					for i in 0..npairs {
						let matc = rdr.read_i32::<BigEndian>()?;
						let jump = (rdr.read_i32::<BigEndian>()? + this_pc as i32) as u32;
						insn.cases.insert(matc, *pc_label_map.get(&jump).ok_or_else(ParserError::unmapped_label)?);
					}
					
					pc += pad + (2 * 4) + (npairs * 2 * 4);
					
					Insn::LookupSwitch(insn)
				}
				InsnParser::LOR => Insn::Or(OrInsn::new(IntegerType::Long)),
				InsnParser::LREM => Insn::Remainder(RemainderInsn::new(PrimitiveType::Long)),
				InsnParser::LRETURN => Insn::Return(ReturnInsn::new(ReturnType::Long)),
				InsnParser::LSHL => Insn::ShiftLeft(ShiftLeftInsn::new(IntegerType::Long)),
				InsnParser::LSHR => Insn::ShiftRight(ShiftRightInsn::new(IntegerType::Long)),
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
				InsnParser::LUSHR => Insn::LogicalShiftRight(LogicalShiftRightInsn::new(IntegerType::Long)),
				InsnParser::LXOR => Insn::Xor(XorInsn::new(IntegerType::Long)),
				InsnParser::MONITORENTER => Insn::MonitorEnter(MonitorEnterInsn::new()),
				InsnParser::MONITOREXIT => Insn::MonitorExit(MonitorExitInsn::new()),
				InsnParser::MULTIANEWARRAY => {
					let kind = constant_pool.utf8(constant_pool.class(rdr.read_u16::<BigEndian>()?)?.name_index)?.str.clone();
					let dimensions = rdr.read_u8()?;
					pc += 3;
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
						4 => Type::Boolean,
						5 => Type::Char,
						6 => Type::Float,
						7 => Type::Double,
						8 => Type::Byte,
						9 => Type::Short,
						10 => Type::Int,
						11 => Type::Long,
						_ => return Err(ParserError::other("Unknown Primitive Type"))
					};
					Insn::NewArray(NewArrayInsn::new(kind))
				},
				InsnParser::NOP => Insn::Nop(NopInsn::new()),
				InsnParser::POP => Insn::Pop(PopInsn::new(false)),
				InsnParser::POP2 => Insn::Pop(PopInsn::new(true)),
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
				InsnParser::SALOAD => Insn::ArrayLoad(ArrayLoadInsn::new(Type::Short)),
				InsnParser::SASTORE => Insn::ArrayStore(ArrayStoreInsn::new(Type::Short)),
				InsnParser::SIPUSH => {
					let short = rdr.read_i16::<BigEndian>()?;
					pc += 2;
					Insn::Ldc(LdcInsn::new(LdcType::Int(short as i32)))
				},
				InsnParser::SWAP => Insn::Swap(SwapInsn::new()),
				InsnParser::TABLESWITCH => {
					let pad = 3 - (this_pc % 4);
					rdr.read_nbytes(pad as usize)?;
					
					let default = (rdr.read_i32::<BigEndian>()? + this_pc as i32) as u32;
					
					let low = rdr.read_i32::<BigEndian>()?;
					let high = rdr.read_i32::<BigEndian>()?;
					let num_cases = (high - low + 1) as u32;
					let mut cases: Vec<LabelInsn> = Vec::with_capacity(num_cases as usize);
					for i in 0..num_cases {
						let case = (rdr.read_i32::<BigEndian>()? + this_pc as i32) as u32;
						cases.push(*pc_label_map.get(&case).ok_or_else(ParserError::unmapped_label)?);
					}
					
					pc += pad + ((3 + num_cases) * 4);
					
					Insn::TableSwitch(TableSwitchInsn {
						default: *pc_label_map.get(&default).ok_or_else(ParserError::unmapped_label)?,
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
						InsnParser::IINC => {
							let index = rdr.read_u16::<BigEndian>()?;
							let amount = rdr.read_i16::<BigEndian>()?;
							pc += 4;
							Insn::IncrementInt(IncrementIntInsn::new(index, amount))
						}
						InsnParser::RET => unimplemented!("Wide Ret instructions are not implemented"),
						_ => return Err(ParserError::invalid_insn(this_pc, format!("Invalid wide opcode {:x}", opcode)))
					}
				}
				_ => return Err(ParserError::unknown_insn(opcode))
			};
			insns.push(insn);
			
			index += 1;
		}
		
		let list = InsnList {
			insns,
			labels: pc_label_map.len() as u32
		};
		
		Ok(list)
	}
	
	fn remap_label_nodes(x: &mut LabelInsn, list: &mut InsnList, pc_index_map: &HashMap<u32, u32>, insert: &mut HashMap<usize, Vec<Insn>>) -> Result<()> {
		let mut insert_into = *match pc_index_map.get(&x.id) {
			Some(x) => x,
			_ => return Err(ParserError::out_of_bounds_jump(x.id as i32))
		};
		
		for (i, insns) in insert.iter() {
			for _ in 0..insns.len() {
				if insert_into as usize > *i {
					insert_into += 1;
				}
			}
		}
		
		let jump_to = list.new_label();
		x.id = jump_to.id;
		
		insert.entry(insert_into as usize)
			.or_insert(Vec::with_capacity(1))
			.push(Insn::Label(jump_to));
		Ok(())
	}
	
	fn parse_ldc(index: CPIndex, constant_pool: &ConstantPool) -> Result<Insn> {
		let constant = constant_pool.get(index)?;
		let ldc_type = match constant {
			ConstantType::String(x) => LdcType::String(constant_pool.utf8(x.utf_index)?.str.clone()),
			ConstantType::Integer(x) => LdcType::Int(x.inner()),
			ConstantType::Float(x) => LdcType::Float(x.inner()),
			ConstantType::Double(x) => LdcType::Double(x.inner()),
			ConstantType::Long(x) => LdcType::Long(x.inner()),
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
	
	fn write_insns(code: &CodeAttribute, constant_pool: &mut ConstantPoolWriter) -> Result<Vec<u8>> {
		let mut wtr: Cursor<Vec<u8>> = Cursor::new(Vec::with_capacity(code.insns.len()));
		
		let mut label_pc_map: HashMap<LabelInsn, u32> = HashMap::new();
		
		enum ReferenceType {
			/// 0: GOTO
			/// 1: indexbyte_1
			/// 2: indexbyte_2
			/// 3: NOP
			/// 4: NOP
			Jump(u32),
			/// 0: OPCODE (IF_IMPEQ, IFEQ...)
			/// 1: indexbyte_1
			/// 2: indexbyte_2
			/// 3: NOP
			/// 4: NOP
			/// 5: NOP
			/// 6: NOP
			/// 7: NOP
			Conditional(u32),
			/// 0: indexbyte_1
			/// 1: indexbyte_2
			/// 2: indexbyte_3
			/// 3: indexbyte_4
			Direct(u32)
		}
		
		let mut forward_references: HashMap<LabelInsn, Vec<ReferenceType>> = HashMap::new();
		
		let mut pc = 0u32;
		for insn in code.insns.iter() {
			match insn {
				Insn::Label(x) => {
					label_pc_map.insert(x.clone(), pc);
					if let Some(refs) = forward_references.get(x) {
						let vec_mut = wtr.get_mut();
						for ref_t in refs.iter() {
							match ref_t {
								ReferenceType::Jump(at) => {
									let i = *at as usize;
									let offset: i32 = pc as i32 - i as i32;
									let off_bytes = offset.to_be_bytes();
									if off_bytes[0] == 0 && off_bytes[1] == 0 {
										vec_mut[i + 1] = off_bytes[2];
										vec_mut[i + 2] = off_bytes[3];
									} else {
										// need to replace with a GOTO_W
										vec_mut[i] = InsnParser::GOTO_W;
										vec_mut[i + 1] = off_bytes[0];
										vec_mut[i + 2] = off_bytes[1];
										vec_mut[i + 3] = off_bytes[2];
										vec_mut[i + 4] = off_bytes[3];
									}
								}
								ReferenceType::Conditional(at) => {
									let i = *at as usize;
									let offset_1: i32 = pc as i32 - i as i32;
									let off_bytes = offset_1.to_be_bytes();
									if off_bytes[0] == 0 && off_bytes[1] == 0 {
										vec_mut[i + 1] = off_bytes[2];
										vec_mut[i + 2] = off_bytes[3];
									} else {
										// need to add a a GOTO_W
										let off_bytes_1 = 3i32.to_be_bytes();
										vec_mut[i + 1] = off_bytes_1[2];
										vec_mut[i + 2] = off_bytes_1[3];
										let offset_2: i32 = pc as i32 - i as i32 - 3;
										let off_bytes_2 = offset_2.to_be_bytes();
										vec_mut[i + 3] = InsnParser::GOTO_W;
										vec_mut[i + 4] = off_bytes_2[0];
										vec_mut[i + 5] = off_bytes_2[1];
										vec_mut[i + 6] = off_bytes_2[2];
										vec_mut[i + 7] = off_bytes_2[3];
									}
								}
								ReferenceType::Direct(at) => {
									let i = *at as usize;
									let offset: i32 = pc as i32 - i as i32;
									let off_bytes = offset.to_be_bytes();
									vec_mut[i + 0] = off_bytes[0];
									vec_mut[i + 1] = off_bytes[1];
									vec_mut[i + 2] = off_bytes[2];
									vec_mut[i + 3] = off_bytes[3];
								}
							}
						}
					}
				}
				Insn::ArrayLoad(x) => {
					wtr.write_u8(match &x.kind {
						Type::Reference(x) => InsnParser::AALOAD,
						Type::Byte | Type::Boolean => InsnParser::BALOAD,
						Type::Char => InsnParser::CALOAD,
						Type::Short => InsnParser::SALOAD,
						Type::Int => InsnParser::IALOAD,
						Type::Long => InsnParser::LALOAD,
						Type::Float => InsnParser::FALOAD,
						Type::Double => InsnParser::DALOAD,
						Type::Void => return Err(ParserError::invalid_insn(pc, "Cannot use type Void in array load"))
					})?;
					pc = pc.checked_add(1).ok_or_else(ParserError::too_many_instructions)?;
				}
				Insn::ArrayStore(x) => {
					wtr.write_u8(match &x.kind {
						Type::Reference(x) => InsnParser::AASTORE,
						Type::Byte | Type::Boolean => InsnParser::BASTORE,
						Type::Char => InsnParser::CASTORE,
						Type::Short => InsnParser::SASTORE,
						Type::Int => InsnParser::IASTORE,
						Type::Long => InsnParser::LASTORE,
						Type::Float => InsnParser::FASTORE,
						Type::Double => InsnParser::DASTORE,
						Type::Void => return Err(ParserError::invalid_insn(pc, "Cannot use type Void in array store"))
					})?;
					pc = pc.checked_add(1).ok_or_else(ParserError::too_many_instructions)?;
				}
				Insn::Ldc(x) => {
					pc = pc.checked_add(match &x.constant {
						LdcType::Null => {
							wtr.write_u8(InsnParser::ACONST_NULL)?;
							1
						}
						LdcType::String(x) => InsnParser::write_ldc(&mut wtr, constant_pool.string_utf(x.clone()), false)?,
						LdcType::Int(x) => InsnParser::write_ldc(&mut wtr, constant_pool.integer(*x), false)?,
						LdcType::Float(x) => InsnParser::write_ldc(&mut wtr, constant_pool.float(*x), false)?,
						LdcType::Long(x) => InsnParser::write_ldc(&mut wtr, constant_pool.long(*x), false)?,
						LdcType::Double(x) => InsnParser::write_ldc(&mut wtr, constant_pool.double(*x), false)?,
						LdcType::Class(x) => InsnParser::write_ldc(&mut wtr, constant_pool.class_utf8(x.clone()), false)?,
						LdcType::MethodType(x) => InsnParser::write_ldc(&mut wtr, constant_pool.methodtype_utf8(x.clone()), false)?,
						LdcType::MethodHandle() => return Err(ParserError::invalid_insn(pc, "MethodHandle LDC")),
						LdcType::Dynamic() => return Err(ParserError::invalid_insn(pc, "Dynamic LDC")),
					}).ok_or_else(ParserError::too_many_instructions)?;
				}
				Insn::LocalLoad(x) => {
					let (op0, op1, op2, op3, opx) = match &x.kind {
						OpType::Reference => (InsnParser::ALOAD_0, InsnParser::ALOAD_1, InsnParser::ALOAD_2, InsnParser::ALOAD_3, InsnParser::ALOAD),
						OpType::Short | OpType::Char | OpType::Byte | OpType::Boolean | OpType::Int => (InsnParser::ILOAD_0, InsnParser::ILOAD_1, InsnParser::ILOAD_2, InsnParser::ILOAD_3, InsnParser::ILOAD),
						OpType::Float => (InsnParser::FLOAD_0, InsnParser::FLOAD_1, InsnParser::FLOAD_2, InsnParser::FLOAD_3, InsnParser::FLOAD),
						OpType::Double => (InsnParser::DLOAD_0, InsnParser::DLOAD_1, InsnParser::DLOAD_2, InsnParser::DLOAD_3, InsnParser::DLOAD),
						OpType::Long => (InsnParser::LLOAD_0, InsnParser::LLOAD_1, InsnParser::LLOAD_2, InsnParser::LLOAD_3, InsnParser::LLOAD),
					};
					match x.index {
						0 => {
							wtr.write_u8(op0)?;
							pc = pc.checked_add(1).ok_or_else(ParserError::too_many_instructions)?;
						}
						1 => {
							wtr.write_u8(op1)?;
							pc = pc.checked_add(1).ok_or_else(ParserError::too_many_instructions)?;
						}
						2 => {
							wtr.write_u8(op2)?;
							pc = pc.checked_add(1).ok_or_else(ParserError::too_many_instructions)?;
						}
						3 => {
							wtr.write_u8(op3)?;
							pc = pc.checked_add(1).ok_or_else(ParserError::too_many_instructions)?;
						}
						index => {
							if index <= 0xFF {
								wtr.write_u8(opx)?;
								wtr.write_u8(index as u8)?;
								pc = pc.checked_add(2).ok_or_else(ParserError::too_many_instructions)?;
							} else {
								wtr.write_u8(InsnParser::WIDE)?;
								wtr.write_u8(opx)?;
								wtr.write_u16::<BigEndian>(index)?;
								pc = pc.checked_add(4).ok_or_else(ParserError::too_many_instructions)?;
							}
						}
					}
				}
				Insn::LocalStore(x) => {
					let (op0, op1, op2, op3, opx) = match &x.kind {
						OpType::Reference => (InsnParser::ASTORE_0, InsnParser::ASTORE_1, InsnParser::ASTORE_2, InsnParser::ASTORE_3, InsnParser::ASTORE),
						OpType::Boolean | OpType::Byte | OpType::Char | OpType::Short | OpType::Int => (InsnParser::ISTORE_0, InsnParser::ISTORE_1, InsnParser::ISTORE_2, InsnParser::ISTORE_3, InsnParser::ISTORE),
						OpType::Float => (InsnParser::FSTORE_0, InsnParser::FSTORE_1, InsnParser::FSTORE_2, InsnParser::FSTORE_3, InsnParser::FSTORE),
						OpType::Double => (InsnParser::DSTORE_0, InsnParser::DSTORE_1, InsnParser::DSTORE_2, InsnParser::DSTORE_3, InsnParser::DSTORE),
						OpType::Long => (InsnParser::LSTORE_0, InsnParser::LSTORE_1, InsnParser::LSTORE_2, InsnParser::LSTORE_3, InsnParser::LSTORE)
					};
					match x.index {
						0 => {
							wtr.write_u8(op0)?;
							pc = pc.checked_add(1).ok_or_else(ParserError::too_many_instructions)?;
						}
						1 => {
							wtr.write_u8(op1)?;
							pc = pc.checked_add(1).ok_or_else(ParserError::too_many_instructions)?;
						}
						2 => {
							wtr.write_u8(op2)?;
							pc = pc.checked_add(1).ok_or_else(ParserError::too_many_instructions)?;
						}
						3 => {
							wtr.write_u8(op3)?;
							pc = pc.checked_add(1).ok_or_else(ParserError::too_many_instructions)?;
						}
						index => {
							if index <= 0xFF {
								wtr.write_u8(opx)?;
								wtr.write_u8(index as u8)?;
								pc = pc.checked_add(2).ok_or_else(ParserError::too_many_instructions)?;
							} else {
								wtr.write_u8(InsnParser::WIDE)?;
								wtr.write_u8(opx)?;
								wtr.write_u16::<BigEndian>(index)?;
								pc = pc.checked_add(4).ok_or_else(ParserError::too_many_instructions)?;
							}
						}
					}
				}
				Insn::NewArray(x) => {
					match &x.kind {
						Type::Reference(x) => {
							let cls = if let Some(cls) = x {
								cls.clone()
							} else {
								// technically this should be invalid and we could throw an error
								// but it's better to just assume the user wants an Object
								String::from("java/lang/Object")
							};
							wtr.write_u8(InsnParser::ANEWARRAY)?;
							wtr.write_u16::<BigEndian>(constant_pool.class_utf8(cls))?;
							pc = pc.checked_add(3).ok_or_else(ParserError::too_many_instructions)?;
						}
						Type::Boolean => {
							wtr.write_u8(InsnParser::NEWARRAY)?;
							wtr.write_u8(4)?;
							pc = pc.checked_add(2).ok_or_else(ParserError::too_many_instructions)?;
						}
						Type::Byte => {
							wtr.write_u8(InsnParser::NEWARRAY)?;
							wtr.write_u8(8)?;
							pc = pc.checked_add(2).ok_or_else(ParserError::too_many_instructions)?;
						}
						Type::Char => {
							wtr.write_u8(InsnParser::NEWARRAY)?;
							wtr.write_u8(5)?;
							pc = pc.checked_add(2).ok_or_else(ParserError::too_many_instructions)?;
						}
						Type::Short => {
							wtr.write_u8(InsnParser::NEWARRAY)?;
							wtr.write_u8(9)?;
							pc = pc.checked_add(2).ok_or_else(ParserError::too_many_instructions)?;
						}
						Type::Int => {
							wtr.write_u8(InsnParser::NEWARRAY)?;
							wtr.write_u8(10)?;
							pc = pc.checked_add(2).ok_or_else(ParserError::too_many_instructions)?;
						}
						Type::Long => {
							wtr.write_u8(InsnParser::NEWARRAY)?;
							wtr.write_u8(11)?;
							pc = pc.checked_add(2).ok_or_else(ParserError::too_many_instructions)?;
						}
						Type::Float => {
							wtr.write_u8(InsnParser::NEWARRAY)?;
							wtr.write_u8(6)?;
							pc = pc.checked_add(2).ok_or_else(ParserError::too_many_instructions)?;
						}
						Type::Double => {
							wtr.write_u8(InsnParser::NEWARRAY)?;
							wtr.write_u8(7)?;
							pc = pc.checked_add(2).ok_or_else(ParserError::too_many_instructions)?;
						},
						Type::Void => return Err(ParserError::invalid_insn(pc, "Cannot use type Void in newarray"))
					}
				}
				Insn::Return(x) => {
					match &x.kind {
						ReturnType::Void => wtr.write_u8(InsnParser::RETURN)?,
						ReturnType::Reference => wtr.write_u8(InsnParser::ARETURN)?,
						// boolean, byte, char and short all use the int return (same size)
						ReturnType::Boolean => wtr.write_u8(InsnParser::IRETURN)?,
						ReturnType::Byte => wtr.write_u8(InsnParser::IRETURN)?,
						ReturnType::Char => wtr.write_u8(InsnParser::IRETURN)?,
						ReturnType::Short => wtr.write_u8(InsnParser::IRETURN)?,
						ReturnType::Int => wtr.write_u8(InsnParser::IRETURN)?,
						ReturnType::Long => wtr.write_u8(InsnParser::LRETURN)?,
						ReturnType::Float => wtr.write_u8(InsnParser::FRETURN)?,
						ReturnType::Double => wtr.write_u8(InsnParser::DRETURN)?,
					}
					pc = pc.checked_add(1).ok_or_else(ParserError::too_many_instructions)?;
				}
				Insn::ArrayLength(x) => {
					wtr.write_u8(InsnParser::ARRAYLENGTH)?;
					pc = pc.checked_add(1).ok_or_else(ParserError::too_many_instructions)?;
				}
				Insn::Throw(x) => {
					wtr.write_u8(InsnParser::ATHROW)?;
					pc = pc.checked_add(1).ok_or_else(ParserError::too_many_instructions)?;
				}
				Insn::CheckCast(x) => {
					wtr.write_u8(InsnParser::CHECKCAST)?;
					wtr.write_u16::<BigEndian>(constant_pool.class_utf8(x.kind.clone()))?;
					pc = pc.checked_add(3).ok_or_else(ParserError::too_many_instructions)?;
				}
				Insn::Convert(x) => {
					match &x.from {
						PrimitiveType::Short | PrimitiveType::Char | PrimitiveType::Boolean | PrimitiveType::Byte | PrimitiveType::Int => {
							wtr.write_u8(match &x.to {
								PrimitiveType::Boolean | PrimitiveType::Byte => InsnParser::I2B,
								PrimitiveType::Char => InsnParser::I2C,
								PrimitiveType::Short => InsnParser::I2S,
								PrimitiveType::Int => InsnParser::NOP,
								PrimitiveType::Long => InsnParser::I2L,
								PrimitiveType::Float => InsnParser::I2F,
								PrimitiveType::Double => InsnParser::I2D
							})?;
							pc = pc.checked_add(1).ok_or_else(ParserError::too_many_instructions)?;
						}
						PrimitiveType::Long => {
							wtr.write_u8(match &x.to {
								PrimitiveType::Short | PrimitiveType::Char | PrimitiveType::Boolean | PrimitiveType::Byte | PrimitiveType::Int => InsnParser::L2I,
								PrimitiveType::Long => InsnParser::NOP,
								PrimitiveType::Float => InsnParser::L2F,
								PrimitiveType::Double => InsnParser::L2D
							})?;
							pc = pc.checked_add(1).ok_or_else(ParserError::too_many_instructions)?;
						}
						PrimitiveType::Float => {
							wtr.write_u8(match &x.to {
								PrimitiveType::Short | PrimitiveType::Char | PrimitiveType::Boolean | PrimitiveType::Byte | PrimitiveType::Int => InsnParser::F2I,
								PrimitiveType::Long => InsnParser::F2L,
								PrimitiveType::Float => InsnParser::NOP,
								PrimitiveType::Double => InsnParser::F2D
							})?;
							pc = pc.checked_add(1).ok_or_else(ParserError::too_many_instructions)?;
						}
						PrimitiveType::Double => {
							wtr.write_u8(match &x.to {
								PrimitiveType::Short | PrimitiveType::Char | PrimitiveType::Boolean | PrimitiveType::Byte | PrimitiveType::Int => InsnParser::D2I,
								PrimitiveType::Long => InsnParser::D2L,
								PrimitiveType::Float => InsnParser::D2F,
								PrimitiveType::Double => InsnParser::NOP
							})?;
							pc = pc.checked_add(1).ok_or_else(ParserError::too_many_instructions)?;
						}
					}
				}
				Insn::Add(x) => {
					wtr.write_u8(match &x.kind {
						PrimitiveType::Boolean => InsnParser::IADD,
						PrimitiveType::Byte => InsnParser::IADD,
						PrimitiveType::Char => InsnParser::IADD,
						PrimitiveType::Short => InsnParser::IADD,
						PrimitiveType::Int => InsnParser::IADD,
						PrimitiveType::Long => InsnParser::LADD,
						PrimitiveType::Float => InsnParser::FADD,
						PrimitiveType::Double => InsnParser::DADD
					})?;
					pc = pc.checked_add(1).ok_or_else(ParserError::too_many_instructions)?;
				}
				Insn::Compare(x) => {
					match &x.kind {
						PrimitiveType::Boolean | PrimitiveType::Byte | PrimitiveType::Char | PrimitiveType::Short | PrimitiveType::Int => {
							// there's no int comparison opcode, but we can use long comparison
							wtr.write_u8(InsnParser::I2L)?;
							wtr.write_u8(InsnParser::LCMP)?;
							pc = pc.checked_add(2).ok_or_else(ParserError::too_many_instructions)?;
						}
						PrimitiveType::Long => {
							wtr.write_u8(InsnParser::LCMP)?;
							pc = pc.checked_add(1).ok_or_else(ParserError::too_many_instructions)?;
						}
						PrimitiveType::Float => {
							wtr.write_u8(if x.pos_on_nan { InsnParser::FCMPG } else { InsnParser::FCMPL })?;
							pc = pc.checked_add(1).ok_or_else(ParserError::too_many_instructions)?;
						}
						PrimitiveType::Double => {
							wtr.write_u8(if x.pos_on_nan { InsnParser::DCMPG } else { InsnParser::DCMPL })?;
							pc = pc.checked_add(1).ok_or_else(ParserError::too_many_instructions)?;
						}
					}
				}
				Insn::Divide(x) => {
					wtr.write_u8(match &x.kind {
						PrimitiveType::Boolean | PrimitiveType::Byte | PrimitiveType::Char | PrimitiveType::Short | PrimitiveType::Int => InsnParser::IDIV,
						PrimitiveType::Long => InsnParser::LDIV,
						PrimitiveType::Float => InsnParser::FDIV,
						PrimitiveType::Double => InsnParser::DDIV
					})?;
					pc = pc.checked_add(1).ok_or_else(ParserError::too_many_instructions)?;
				}
				Insn::Multiply(x) => {
					wtr.write_u8(match &x.kind {
						PrimitiveType::Boolean | PrimitiveType::Byte | PrimitiveType::Char | PrimitiveType::Short | PrimitiveType::Int => InsnParser::IMUL,
						PrimitiveType::Long => InsnParser::LMUL,
						PrimitiveType::Float => InsnParser::FMUL,
						PrimitiveType::Double => InsnParser::DMUL
					})?;
					pc = pc.checked_add(1).ok_or_else(ParserError::too_many_instructions)?;
				}
				Insn::Negate(x) => {
					wtr.write_u8(match &x.kind {
						PrimitiveType::Boolean | PrimitiveType::Byte | PrimitiveType::Char | PrimitiveType::Short | PrimitiveType::Int => InsnParser::INEG,
						PrimitiveType::Long => InsnParser::LNEG,
						PrimitiveType::Float => InsnParser::FNEG,
						PrimitiveType::Double => InsnParser::DNEG
					})?;
					pc = pc.checked_add(1).ok_or_else(ParserError::too_many_instructions)?;
				}
				Insn::Remainder(x) => {
					wtr.write_u8(match &x.kind {
						PrimitiveType::Boolean | PrimitiveType::Byte | PrimitiveType::Char | PrimitiveType::Short | PrimitiveType::Int => InsnParser::IREM,
						PrimitiveType::Long => InsnParser::LREM,
						PrimitiveType::Float => InsnParser::FREM,
						PrimitiveType::Double => InsnParser::DREM
					})?;
					pc = pc.checked_add(1).ok_or_else(ParserError::too_many_instructions)?;
				}
				Insn::Subtract(x) => {
					wtr.write_u8(match &x.kind {
						PrimitiveType::Boolean | PrimitiveType::Byte | PrimitiveType::Char | PrimitiveType::Short | PrimitiveType::Int => InsnParser::ISUB,
						PrimitiveType::Long => InsnParser::LSUB,
						PrimitiveType::Float => InsnParser::FSUB,
						PrimitiveType::Double => InsnParser::DSUB
					})?;
					pc = pc.checked_add(1).ok_or_else(ParserError::too_many_instructions)?;
				}
				Insn::And(x) => {
					wtr.write_u8(match &x.kind {
						IntegerType::Int => InsnParser::IAND,
						IntegerType::Long => InsnParser::LAND
					})?;
					pc = pc.checked_add(1).ok_or_else(ParserError::too_many_instructions)?;
				}
				Insn::Or(x) => {
					wtr.write_u8(match &x.kind {
						IntegerType::Int => InsnParser::IOR,
						IntegerType::Long => InsnParser::LOR
					})?;
					pc = pc.checked_add(1).ok_or_else(ParserError::too_many_instructions)?;
				}
				Insn::Xor(x) => {
					wtr.write_u8(match &x.kind {
						IntegerType::Int => InsnParser::IXOR,
						IntegerType::Long => InsnParser::LXOR
					})?;
					pc = pc.checked_add(1).ok_or_else(ParserError::too_many_instructions)?;
				}
				Insn::ShiftLeft(x) => {
					wtr.write_u8(match &x.kind {
						IntegerType::Int => InsnParser::ISHL,
						IntegerType::Long => InsnParser::LSHL
					})?;
					pc = pc.checked_add(1).ok_or_else(ParserError::too_many_instructions)?;
				}
				Insn::ShiftRight(x) => {
					wtr.write_u8(match &x.kind {
						IntegerType::Int => InsnParser::ISHR,
						IntegerType::Long => InsnParser::LSHR
					})?;
					pc = pc.checked_add(1).ok_or_else(ParserError::too_many_instructions)?;
				}
				Insn::LogicalShiftRight(x) => {
					wtr.write_u8(match &x.kind {
						IntegerType::Int => InsnParser::IUSHR,
						IntegerType::Long => InsnParser::LUSHR
					})?;
					pc = pc.checked_add(1).ok_or_else(ParserError::too_many_instructions)?;
				}
				Insn::Dup(x) => {
					wtr.write_u8(match x.num {
						1 => {
							match x.down {
								0 => InsnParser::DUP,
								1 => InsnParser::DUP_X1,
								2 => InsnParser::DUP_X2,
								_ => return Err(ParserError::invalid_insn(pc, "DupInsn::down must not be larger than 2"))
							}
						}
						2 => {
							match x.down {
								0 => InsnParser::DUP2,
								1 => InsnParser::DUP2_X1,
								2 => InsnParser::DUP2_X2,
								_ => return Err(ParserError::invalid_insn(pc, "DupInsn::down must not be larger than 2"))
							}
						}
						_ => return Err(ParserError::invalid_insn(pc, "DupInsn::num must be in the range 1-2"))
					})?;
					pc = pc.checked_add(1).ok_or_else(ParserError::too_many_instructions)?;
				}
				Insn::Pop(x) => {
					wtr.write_u8(match x.pop_two {
						false => InsnParser::POP,
						true => InsnParser::POP2,
					})?;
					pc = pc.checked_add(1).ok_or_else(ParserError::too_many_instructions)?;
				}
				Insn::GetField(x) => {
					wtr.write_u8(if x.instance { InsnParser::GETFIELD } else { InsnParser::GETSTATIC })?;
					let class_ref = constant_pool.class_utf8(x.class.clone());
					let name_ref = constant_pool.utf8(x.name.clone());
					let desc_ref = constant_pool.utf8(x.descriptor.clone());
					let nametype_ref = constant_pool.nameandtype(name_ref, desc_ref);
					wtr.write_u16::<BigEndian>(constant_pool.fieldref(class_ref, nametype_ref))?;
					pc = pc.checked_add(3).ok_or_else(ParserError::too_many_instructions)?;
				}
				Insn::PutField(x) => {
					wtr.write_u8(if x.instance { InsnParser::PUTFIELD } else { InsnParser::PUTSTATIC })?;
					let class_ref = constant_pool.class_utf8(x.class.clone());
					let name_ref = constant_pool.utf8(x.name.clone());
					let desc_ref = constant_pool.utf8(x.descriptor.clone());
					let nametype_ref = constant_pool.nameandtype(name_ref, desc_ref);
					wtr.write_u16::<BigEndian>(constant_pool.fieldref(class_ref, nametype_ref))?;
					pc = pc.checked_add(3).ok_or_else(ParserError::too_many_instructions)?;
				}
				Insn::Jump(x) => {
					if let Some(to) = label_pc_map.get(&x.jump_to) {
						let offset: i32 = pc as i32 - (*to) as i32;
						let off_bytes = offset.to_be_bytes();
						// backwards reference
						if off_bytes[0] == 0 && off_bytes[1] == 0 {
							wtr.write_u8(InsnParser::GOTO)?;
							wtr.write_i16::<BigEndian>(offset as i16)?;
							pc = pc.checked_add(3).ok_or_else(ParserError::too_many_instructions)?;
						} else {
							wtr.write_u8(InsnParser::GOTO_W)?;
							wtr.write_i32::<BigEndian>(offset)?;
							pc = pc.checked_add(5).ok_or_else(ParserError::too_many_instructions)?;
						}
					} else {
						if let Some(vec) = forward_references.get_mut(&x.jump_to) {
							vec.push(ReferenceType::Jump(pc));
						} else {
							let mut vec = Vec::new();
							vec.push(ReferenceType::Jump(pc));
							forward_references.insert(x.jump_to.clone(), vec);
						}
						wtr.write_u8(InsnParser::GOTO)?;
						wtr.write_u16::<BigEndian>(0)?;
						wtr.write_u8(InsnParser::NOP)?;
						wtr.write_u8(InsnParser::NOP)?;
						pc = pc.checked_add(8).ok_or_else(ParserError::too_many_instructions)?;
					}
				}
				Insn::ConditionalJump(x) => {
					let opcode = match x.condition {
						JumpCondition::IsNull => InsnParser::IFNULL,
						JumpCondition::NotNull => InsnParser::IFNONNULL,
						JumpCondition::ReferencesEqual => InsnParser::IF_ACMPEQ,
						JumpCondition::ReferencesNotEqual => InsnParser::IF_ACMPNE,
						JumpCondition::IntsEq => InsnParser::IF_ICMPEQ,
						JumpCondition::IntsNotEq => InsnParser::IF_ICMPNE,
						JumpCondition::IntsLessThan => InsnParser::IF_ICMPLT,
						JumpCondition::IntsLessThanOrEq => InsnParser::IF_ICMPLE,
						JumpCondition::IntsGreaterThan => InsnParser::IF_ICMPGT,
						JumpCondition::IntsGreaterThanOrEq => InsnParser::IF_ICMPGE,
						JumpCondition::IntEqZero => InsnParser::IFEQ,
						JumpCondition::IntNotEqZero => InsnParser::IFNE,
						JumpCondition::IntLessThanZero => InsnParser::IFLT,
						JumpCondition::IntLessThanOrEqZero => InsnParser::IFLE,
						JumpCondition::IntGreaterThanZero => InsnParser::IFGT,
						JumpCondition::IntGreaterThanOrEqZero => InsnParser::IFGE
					};
					
					if let Some(to) = label_pc_map.get(&x.jump_to) {
						let offset: i32 = pc as i32 - (*to) as i32;
						let off_bytes = offset.to_be_bytes();
						// backwards reference
						if off_bytes[0] == 0 && off_bytes[1] == 0 {
							wtr.write_u8(opcode)?;
							wtr.write_i16::<BigEndian>(offset as i16)?;
							pc = pc.checked_add(3).ok_or_else(ParserError::too_many_instructions)?;
						} else {
							wtr.write_u8(opcode)?;
							wtr.write_u16::<BigEndian>(3)?;
							wtr.write_u8(InsnParser::GOTO_W)?;
							wtr.write_i32::<BigEndian>(offset - 3)?;
							pc = pc.checked_add(8).ok_or_else(ParserError::too_many_instructions)?;
						}
					} else {
						if let Some(vec) = forward_references.get_mut(&x.jump_to) {
							vec.push(ReferenceType::Conditional(pc));
						} else {
							let mut vec = Vec::new();
							vec.push(ReferenceType::Conditional(pc));
							forward_references.insert(x.jump_to.clone(), vec);
						}
						wtr.write_u8(opcode)?;
						wtr.write_u16::<BigEndian>(0)?;
						wtr.write_u8(InsnParser::NOP)?;
						wtr.write_u8(InsnParser::NOP)?;
						wtr.write_u8(InsnParser::NOP)?;
						wtr.write_u8(InsnParser::NOP)?;
						wtr.write_u8(InsnParser::NOP)?;
						pc = pc.checked_add(8).ok_or_else(ParserError::too_many_instructions)?;
					}
				}
				Insn::IncrementInt(x) => {
					let index = x.index;
					let amount = x.amount;
					// need to check if we can fit the amount into 1 byte
					if let (Ok(index), Ok(amount)) = (u8::try_from(index), i8::try_from(amount)) {
						wtr.write_u8(InsnParser::IINC)?;
						wtr.write_u8(index)?;
						wtr.write_i8(amount)?;
						pc = pc.checked_add(3).ok_or_else(ParserError::too_many_instructions)?;
					} else {
						wtr.write_u8(InsnParser::WIDE)?;
						wtr.write_u8(InsnParser::IINC)?;
						wtr.write_u16::<BigEndian>(index)?;
						wtr.write_i16::<BigEndian>(amount)?;
						pc = pc.checked_add(6).ok_or_else(ParserError::too_many_instructions)?;
					}
				}
				Insn::InstanceOf(x) => {
					wtr.write_u8(InsnParser::INSTANCEOF)?;
					wtr.write_u16::<BigEndian>(constant_pool.class_utf8(x.class.clone()))?;
					pc = pc.checked_add(3).ok_or_else(ParserError::too_many_instructions)?;
				}
				Insn::InvokeDynamic(x) => {
					return Err(ParserError::unimplemented("Invokedynamic writing unimplemented"));
				}
				Insn::Invoke(x) => {
					let opcode = match x.kind {
						InvokeType::Instance => InsnParser::INVOKEVIRTUAL,
						InvokeType::Static => InsnParser::INVOKESTATIC,
						InvokeType::Interface => InsnParser::INVOKEINTERFACE,
						InvokeType::Special => InsnParser::INVOKESPECIAL
					};
					wtr.write_u8(opcode)?;
					if opcode == InsnParser::INVOKEINTERFACE {
						let class = constant_pool.class_utf8(x.class.clone());
						let name = constant_pool.utf8(x.name.clone());
						let desc = constant_pool.utf8(x.descriptor.clone());
						let nandt = constant_pool.nameandtype(name, desc);
						wtr.write_u16::<BigEndian>(constant_pool.interfacemethodref(class, nandt))?;
						// The count operand of an invokeinterface instruction is valid if it is
						// the difference between the size of the operand stack before and after the instruction
						// executes.
						let mut count = 1; // interface methods are virtual so there is always at least one
						let (args, _) = parse_method_desc(&x.descriptor)?;
						for arg in args.iter() {
							count += arg.size();
						}
						wtr.write_u8(count)?;
						wtr.write_u8(0)?;
						pc = pc.checked_add(5).ok_or_else(ParserError::too_many_instructions)?;
					} else {
						let class = constant_pool.class_utf8(x.class.clone());
						let name = constant_pool.utf8(x.name.clone());
						let desc = constant_pool.utf8(x.descriptor.clone());
						let nandt = constant_pool.nameandtype(name, desc);
						wtr.write_u16::<BigEndian>(constant_pool.methodref(class, nandt))?;
						pc = pc.checked_add(3).ok_or_else(ParserError::too_many_instructions)?;
					}
				}
				Insn::LookupSwitch(x) => {
					wtr.write_u8(InsnParser::LOOKUPSWITCH)?;
					let pad = (4 - (pc % 4)) % 4;
					for i in 0..pad {
						wtr.write_u8(0)?;
					}
					
					if let Some(at) = label_pc_map.get(&x.default) {
						let offset: i32 = pc as i32 - (*at) as i32;
						wtr.write_i32::<BigEndian>(offset)?;
					} else {
						if let Some(vec) = forward_references.get_mut(&x.default) {
							vec.push(ReferenceType::Direct(pc + 2));
						} else {
							let mut vec = Vec::new();
							vec.push(ReferenceType::Direct(pc + 2));
							forward_references.insert(x.default.clone(), vec);
						}
						wtr.write_i32::<BigEndian>(0)?;
					}
					
					wtr.write_i32::<BigEndian>(x.cases.len() as i32)?;
					
					pc = pc.checked_add(10).ok_or_else(ParserError::too_many_instructions)?;
					
					for (case, to) in x.cases.iter() {
						wtr.write_i32::<BigEndian>(*case)?;
						if let Some(at) = label_pc_map.get(to) {
							let offset: i32 = (pc + 4) as i32 - (*at) as i32;
							wtr.write_i32::<BigEndian>(offset)?;
						} else {
							if let Some(vec) = forward_references.get_mut(to) {
								vec.push(ReferenceType::Direct(pc + 4));
							} else {
								let mut vec = Vec::new();
								vec.push(ReferenceType::Direct(pc + 4));
								forward_references.insert(to.clone(), vec);
							}
							wtr.write_i32::<BigEndian>(0)?;
						}
						pc = pc.checked_add(8).ok_or_else(ParserError::too_many_instructions)?;
					}
				}
				Insn::TableSwitch(x) => {
					wtr.write_u8(InsnParser::TABLESWITCH)?;
					let pad = (4 - (pc % 4)) % 4;
					for i in 0..pad {
						wtr.write_u8(0)?;
					}
				}
				Insn::MonitorEnter(_) => {}
				Insn::MonitorExit(_) => {}
				Insn::MultiNewArray(_) => {}
				Insn::NewObject(_) => {}
				Insn::Nop(_) => {}
				Insn::Swap(_) => {}
				Insn::ImpDep1(_) => {}
				Insn::ImpDep2(_) => {}
				Insn::BreakPoint(_) => {}
			}
		}
		
		Ok(wtr.into_inner())
	}
	
	fn write_ldc<T: Write>(wtr: &mut T, constant: u16, double_size: bool) -> Result<u32> {
		// double sized constants must use LDC2 (only wide variant exists)
		if double_size {
			wtr.write_u8(InsnParser::LDC2_W)?;
			wtr.write_u16::<BigEndian>(constant)?;
			Ok(5)
		} else {
			// If we can fit the constant index into a u8 then use LDC otherwise use LDC_W
			if constant <= 0xFF {
				wtr.write_u8(InsnParser::LDC)?;
				wtr.write_u8(constant as u8)?;
				Ok(3)
			} else {
				wtr.write_u8(InsnParser::LDC_W)?;
				wtr.write_u16::<BigEndian>(constant)?;
				Ok(5)
			}
		}
	}
}
