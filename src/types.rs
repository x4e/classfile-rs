use crate::error::{Result, ParserError};

const VOID: char = 'V';
const BYTE: char = 'B';
const CHAR: char = 'C';
const DOUBLE: char = 'D';
const FLOAT: char = 'F';
const INT: char = 'I';
const LONG: char = 'J';
const SHORT: char = 'S';
const BOOLEAN: char = 'Z';

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Type {
	Reference(Option<String>), // If None then the reference refers to no particular class
	Boolean,
	Byte,
	Char,
	Short,
	Int,
	Long,
	Float,
	Double,
	Void
}

impl Type {
	/// returns the size of the type as a multiple of a dword
	pub fn size(&self) -> u8 {
		match self {
			Type::Reference(_) => 1,
			Type::Boolean => 1,
			Type::Byte => 1,
			Type::Char => 1,
			Type::Short => 1,
			Type::Int => 1,
			Type::Long => 2,
			Type::Float => 1,
			Type::Double => 2,
			Type::Void => 0,
		}
	}
}

pub fn parse_method_desc(desc: &String) -> Result<(Vec<Type>, Type)> {
	parse_method_desc_chars(&desc.as_bytes())
}

fn parse_method_desc_chars(desc: &[u8]) -> Result<(Vec<Type>, Type)> {
	if desc[0] != '(' as u8 {
		return Err(ParserError::invalid_descriptor("Method desc must start with '('"));
	}
	let mut args: Vec<Type> = Vec::new();
	let mut i = 1usize;
	while desc[i] != ')' as u8 {
		let (typ, i2) = parse_type_chars(desc, i)?;
		args.push(typ);
		i = i2;
		
		if i >= desc.len() {
			return Err(ParserError::invalid_descriptor("Method desc must have ')'"));
		}
	}
	let (ret, _) = parse_type_chars(desc, i + 1)?;
	Ok((args, ret))
}

pub fn parse_type(desc: &String) -> Result<(Type, usize)> {
	parse_type_chars(&desc.as_bytes(), 0)
}

fn parse_type_chars(desc: &[u8], mut index: usize) -> Result<(Type, usize)> {
	if index == desc.len() {
		return Err(ParserError::invalid_descriptor("Empty type string"));
	}
	Ok(match desc[index] as char {
		VOID => (Type::Void, index + 1),
		BYTE => (Type::Byte, index + 1),
		CHAR => (Type::Char, index + 1),
		DOUBLE => (Type::Double, index + 1),
		FLOAT => (Type::Float, index + 1),
		INT => (Type::Int, index + 1),
		LONG => (Type::Long, index + 1),
		SHORT => (Type::Short, index + 1),
		BOOLEAN => (Type::Boolean, index + 1),
		'L' => {
			let mut buf = String::new();
			while desc[index] != ';' as u8 {
				index += 1;
				if index >= desc.len() {
					return Err(ParserError::invalid_descriptor("Type missing ';'"))
				}
				buf.push(desc[index] as char);
			}
			(Type::Reference(Some(buf)), index + 1)
		}
		x => return Err(ParserError::invalid_descriptor(format!("Unknown type '{}'", x)))
	})
}
