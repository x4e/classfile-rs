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

pub fn parse_method_desc(desc: String) -> Result<(Vec<Type>, Type)> {
	if desc[0] != '(' {
		return Err(ParserError::invalid_descriptor("Method desc must start with '('"));
	}
	let mut args: Vec<Type> = Vec::new();
	let mut i = 1usize;
	while desc[i] != ')' {
		let (typ, i2) = parse_type(&desc, i)?;
		args.push(typ);
		i = i2;
		
		i += 1;
		if i >= desc.len() {
			return Err(ParserError::invalid_descriptor("Method desc must have ')'"));
		}
	}
	let (ret, i2) = parse_type(&desc, i)?;
	Ok((args, ret))
}

pub fn parse_type(desc: &String, mut index: usize) -> Result<(Type, usize)> {
	if index == desc.len() {
		return Err(ParserError::invalid_descriptor("Empty type string"));
	}
	Ok(match desc[index] {
		VOID => (Type::Void, 1),
		BYTE => (Type::Byte, 1),
		CHAR => (Type::Char, 1),
		DOUBLE => (Type::Double, 1),
		FLOAT => (Type::Float, 1),
		INT => (Type::Int, 1),
		LONG => (Type::Long, 1),
		SHORT => (Type::Short, 1),
		BOOLEAN => (Type::Boolean, 1),
		'L' => {
			let mut buf = String::new();
			while desc[index] != ';' {
				index += 1;
				if index >= desc.len() {
					return Err(ParserError::invalid_descriptor("Type missing ';'"))
				}
				buf.push(desc[index]);
			}
			(Type::Reference(Some(buf)), index + 1)
		}
		x => return Err(ParserError::invalid_descriptor(format!("Unknown type '{}'", x)))
	})
}
