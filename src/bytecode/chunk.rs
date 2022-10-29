use core::ops::Index;
use std::{cell::RefCell, mem::size_of};

use crate::bytecode::prelude::*;

#[derive(Clone)]
pub enum Value<'source> {
	Number(f64),
	Bool(bool),
	Null,
	Obj(ObjRef),
	StrRef(&'source str),
}

impl core::fmt::Debug for Value<'_> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Value::Number(n) => write!(f, "{}", n),
			Value::Bool(v) => write!(f, "{}", v),
			Value::Null => write!(f, "null"),
			Value::Obj(s) => write!(f, "{:?}", s),
			Value::StrRef(s) => write!(f, "\"{s}\""),
		}
	}
}

impl PartialEq for Value<'_> {
	fn eq(&self, other: &Self) -> bool {
		match (self, other) {
			(Self::Number(l0), Self::Number(r0)) => l0 == r0,
			(Self::Bool(l0), Self::Bool(r0)) => l0 == r0,
			(Self::Obj(l0), Self::Obj(r0)) => {
				l0.object_ty() == r0.object_ty()
					&& match l0.object_ty() {
						ObjTy::Str => l0 == r0,
						ObjTy::Other => unimplemented!(),
					}
			}

			(Self::StrRef(l0), Self::StrRef(r0)) => l0 == r0,
			(Self::StrRef(l0), Self::Obj(objref)) | (Self::Obj(objref), Self::StrRef(l0)) => matches!(objref.as_ref::<String>(), l0),
			(Self::Null, Self::Null) => true,
			_ => false,
		}
	}
}

/// Contains a seiries of bytecode instructions along with associated constants and [Line] numbers.
#[derive(Default, Debug)]
pub struct Chunk<'source> {
	code: Vec<u8>,
	constants: Vec<Value<'source>>,

	/// The line numbers, one for each line of bytecode.
	pub lines: Vec<Line>,
}

impl<'source> Chunk<'source> {
	/// Construct an empty chunk
	pub fn new() -> Self {
		Self::default()
	}
	/// Push a byte to the bytecode
	#[inline]
	pub fn push(&mut self, code: impl Into<u8>, line: Line) {
		self.code.push(code.into());
		self.lines.push(line);
	}
	/// Length of bytecode
	#[inline]
	pub fn len(&self) -> usize {
		self.code.len()
	}
	/// Push a constant.
	///
	/// First inserts either a [`Opcode::Constant`] or [`Opcode::LongConstant`] depending on the current number of constants,
	/// then it inserts the constant index, a single byte for normal constants and three bytes for long constants.
	/// It also pushes the constant into the chunk's storage.
	pub fn push_constant(&mut self, constant: Value<'source>, line: Line) {
		self.constants.push(constant);
		let id = self.constants.len() - 1;
		if id <= u8::MAX as usize {
			self.push(Opcode::Constant, line);
			self.push(id as u8, line);
		} else {
			self.push(Opcode::LongConstant, line);
			self.push((id >> 16) as u8, line);
			self.push((id >> 8) as u8, line);
			self.push(id as u8, line);
		}
	}
	/// Retrieves a constant by index (unchecked).
	#[inline]
	pub fn constant(&self, idx: usize) -> &Value<'source> {
		&self.constants[idx]
	}

	/// Gets a raw pointer to the start of the bytecode.
	#[inline]
	pub fn as_ptr(&self) -> *const u8 {
		self.code.as_ptr()
	}
}

impl<'source> Index<usize> for Chunk<'source> {
	type Output = u8;

	/// Fast indexing for the Chunk's bytecode.
	#[inline]
	fn index(&self, index: usize) -> &Self::Output {
		unsafe { &*self.code.as_ptr().offset(index as isize) }
	}
}

/// Disassembles the chunk, with the specified user facing name
#[cfg(feature = "trace_execution")]
#[macro_export]
macro_rules! disassemble {
	(chunk = $chunk:expr, name = $name:expr) => {
		trace!("==== {} ====", $name);
		let mut offset = 0;
		while offset < $chunk.len() {
			offset = disassemble_instruction($chunk, offset);
		}
	};
}
