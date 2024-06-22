use core::ops::Index;
use std::{cell::RefCell, mem::size_of, sync::Arc};

use crate::bytecode::prelude::*;

#[derive(Clone, Copy)]
pub enum Value {
	Number(f64),
	Bool(bool),
	Null,
	Obj(ObjRef),
}

impl core::fmt::Debug for Value {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
			Value::Number(n) => write!(f, "{}", n),
			Value::Bool(v) => write!(f, "{}", v),
			Value::Null => write!(f, "null"),
			Value::Obj(s) => write!(f, "{:?}", s),
		}
	}
}

impl PartialEq for Value {
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
			(Self::Null, Self::Null) => true,
			_ => false,
		}
	}
}

/// Contains a seiries of bytecode instructions along with associated constants and [Line] numbers.
#[derive(Default, Debug)]
pub struct Chunk {
	pub code: Vec<u8>,
	constants: Vec<Value>,
	pub strings: Vec<ObjRef>,
	pub objects: Vec<Box<ObjTy>>,

	/// The line numbers, one for each line of bytecode.
	pub lines: Vec<Line>,
}

impl Chunk {
	pub const EMPTY: Self = Self {
		code: Vec::new(),
		constants: Vec::new(),
		strings: Vec::new(),
		objects: Vec::new(),
		lines: Vec::new(),
	};

	/// Construct an empty chunk
	pub const fn new() -> Self {
		Self::EMPTY
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

	/// Makes a constant in the chunk's storage, returning the index of the constant
	pub fn make_constant(&mut self, constant: Value) -> usize {
		self.constants.push(constant);
		self.constants.len() - 1
	}

	pub fn make_string(&mut self, val: String) -> usize {
		let (reference, obj) = ObjRef::new(val);
		self.objects.push(obj);
		self.strings.push(reference);
		self.make_constant(Value::Obj(reference))
	}

	/// Push a constant.
	///
	/// First inserts either a the `short_op` or `long_op` depending on the current number of constants,
	/// then it inserts the constant index, a single byte for normal constants and three bytes for long constants.
	pub fn push_constant(&mut self, id: usize, line: Line, short_op: Opcode, long_op: Opcode) {
		if id <= u8::MAX as usize {
			self.push(short_op, line);
			self.push(id as u8, line);
		} else {
			self.push(long_op, line);
			self.push((id >> 16) as u8, line);
			self.push((id >> 8) as u8, line);
			self.push(id as u8, line);
		}
	}
	/// Retrieves a constant by index (unchecked).
	#[inline]
	pub fn constant(&self, idx: usize) -> &Value {
		&self.constants[idx]
	}

	/// Gets a raw pointer to the start of the bytecode.
	#[inline]
	pub fn as_ptr(&self) -> *const u8 {
		self.code.as_ptr()
	}
}

impl<'source> Index<usize> for Chunk {
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
