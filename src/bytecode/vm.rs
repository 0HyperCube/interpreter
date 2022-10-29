use std::fmt::Arguments;

use ahash::{AHashMap, AHashSet};

use crate::bytecode::prelude::*;

macro_rules! runtime_error {
	($runtime:ident, $($arg:tt)+) => {
		let line = $runtime.chunk.lines[$runtime.offset()];
		error!(target: "nonew", $($arg)+);
		println!("[line {line}] in script");
		$runtime.reset_stack();
	};
}

/// The interpeter's runtime, containing the current [Chunk], a pointer to the next instruction and the stack
pub struct Runtime<'a, 'source> {
	/// The [`Chunk`] that is being interpreted
	chunk: &'a Chunk<'source>,
	/// The instruction pointer, pointing to the next instruction
	ip: *const u8,
	/// The stack of values that can be pushed to and popped from
	stack: Vec<Value<'source>>,
	/// Pointer to the top of the stack (leading to slightly better performance)
	stack_top: *mut Value<'source>,
	/// All the heap objects need to be stored so they can be deleted by garbage collection
	objects: Vec<Box<ObjTy>>,
	/// A hash table of all strings (to reduce memory usage and comparison times)
	strings: AHashSet<ObjRef>,
}

impl<'a, 'source> Runtime<'a, 'source> {
	/// Construct a new runtime with the specified [Chunk]
	pub fn new(chunk: &'a Chunk<'source>) -> Self {
		let mut stack = Vec::with_capacity(5);
		Self {
			chunk: &chunk,
			ip: chunk.as_ptr(),
			stack_top: stack.as_mut_ptr(),
			stack,
			objects: Vec::new(),
			strings: Default::default(),
		}
	}

	/// Reset Runtime and load new chunk
	pub fn reset(&mut self, chunk: &'a Chunk<'source>) {
		self.chunk = chunk;
		self.ip = chunk.as_ptr();
		self.reset_stack();
		self.free_objects();
		self.strings.clear();
	}

	/// Clear the stack and reset the stack top
	pub fn reset_stack(&mut self) {
		self.stack.clear();
		self.stack_top = self.stack.as_mut_ptr();
	}

	/// Allocates a new string object, using string interning for cheaper comparsions
	///
	/// Note: strings are immutable
	pub fn new_string(&mut self, val: String) -> ObjRef {
		self.strings.iter().copied().find(|existing_str| existing_str.as_ref_unchecked::<String>() == &val).unwrap_or_else(|| {
			let (obj_ref, owned) = ObjRef::new(val);
			self.objects.push(owned);
			self.strings.insert(obj_ref);
			obj_ref
		})
	}

	/// Read a byte of bytecode and move to the next one
	#[inline]
	pub fn read_byte(&mut self) -> u8 {
		unsafe {
			let result = *self.ip;
			self.ip = self.ip.offset(1);
			result
		}
	}

	/// Read a short constant from the [Chunk].
	#[inline]
	pub fn short_constant(&mut self) -> &'a Value<'source> {
		self.chunk.constant(self.read_byte() as usize)
	}

	/// Read a long constant from the [Chunk].
	#[inline]
	pub fn long_constant(&mut self) -> &'a Value<'source> {
		let mut constant_idx = 0;
		for i in 0..3 {
			constant_idx <<= 8;
			constant_idx ^= self.read_byte() as usize;
		}
		self.chunk.constant(constant_idx)
	}

	/// Find the current offset (in bytes) from the start of the chunk to the instruction pointer
	#[cfg(feature = "trace_execution")]
	fn offset(&self) -> usize {
		(unsafe { self.ip.offset_from(self.chunk.as_ptr()) }) as usize
	}

	/// Push an item to the top of the stack
	#[inline]
	pub fn push_stack(&mut self, value: Value<'source>) {
		unsafe {
			// Update stack size
			self.stack.set_len(self.stack.as_ptr().offset_from(self.stack_top) as usize);
			*self.stack_top = value;
			self.stack_top = self.stack_top.offset(1);
		}
	}
	/// Pops an item from the top of the stack, returning it
	#[inline]
	pub fn pop_stack(&mut self) -> &'a Value<'source> {
		unsafe {
			self.stack_top = self.stack_top.offset(-1);
			&*self.stack_top
		}
	}

	/// Peeks at an item a certain distance from the top of the stack
	#[inline]
	pub fn peep_stack(&mut self, distance: isize) -> &'a Value<'source> {
		unsafe { &*self.stack_top.offset(-distance) }
	}

	// /// Allocates an object, storing it in the objects list so it can be garbage collected. Returns a raw pointer to the object.
	// #[inline]
	// pub fn allocate_obj(&mut self, obj: impl Into<ObjTy>) -> *mut ObjTy {
	// 	self.objects.push(obj.into());
	// 	unsafe { self.objects.as_mut_ptr_range().end.offset(-1) }
	// }

	/// Removes all heap allocated objects (do not leave references to these objects)
	#[inline]
	fn free_objects(&mut self) {
		while let Some(obj) = self.objects.pop() {
			ObjTy::free(obj)
		}
	}

	/// Interprets the [Chunk], matching each opcode instruction.
	pub fn interpret(&mut self) -> Result<(), InterpretError> {
		trace!("Interpreting chunk");
		assert_ne!(self.chunk.len(), 0, "Chunk should not be empty");
		loop {
			#[cfg(feature = "trace_execution")]
			{
				let mut current = self.stack.as_ptr();

				if current != self.stack_top {
					trace!(target: "Stack", "");
					while current != self.stack_top {
						unsafe {
							print!("[ {:?} ]", *current);
							current = current.offset(1);
						}
					}
					println!();
				}

				disassemble_instruction(self.chunk, self.offset());
			}

			let instruction = self.read_byte();
			let opcode = instruction.into();

			macro_rules! binary_op {
				($op:tt => $resultv:tt) => {
					{
						let b = self.pop_stack();
						let a = self.pop_stack();
						if let [Value::Number(a), Value::Number(b)] = [a,b]{
							self.push_stack(Value::$resultv(a $op b));
						}else{
							runtime_error!(self, "Operands must be numbers");
						}

					}
				};
			}

			match opcode {
				Opcode::Unknown => warn!("Unknown opcode"),

				Opcode::Constant => {
					let constant = self.short_constant();
					self.push_stack(constant.clone());
				}
				Opcode::LongConstant => {
					let constant = self.long_constant();
					self.push_stack(constant.clone());
				}
				Opcode::Return => return Ok(()),
				Opcode::Negate => {
					let input = self.pop_stack();
					if let Value::Number(input) = input {
						self.push_stack(Value::Number(-input));
					} else {
						runtime_error!(self, "Operands must be numbers");
					}
				}
				Opcode::Add => {
					fn get_str<'a>(b: &'a Value) -> Option<&'a str> {
						match b {
							Value::StrRef(x) => Some(*x),
							Value::Obj(x) => x.as_ref::<String>().map(|x| x.as_str()),
							_ => None,
						}
					}

					let b = self.pop_stack();
					let a = self.pop_stack();
					if let [Value::Number(a), Value::Number(b)] = [a, b] {
						self.push_stack(Value::Number(a + b));
					} else if let Some(b) = get_str(b) && let Some(a) = get_str(a){
						let obj_ref = self.new_string(a.to_string() + b);
						self.push_stack(Value::Obj( obj_ref));
					} else{
						runtime_error!(self, "Operands to '+' must be numbers or strings");
					}
				}
				Opcode::Subtract => binary_op!(- => Number),
				Opcode::Multiply => binary_op!(* => Number),
				Opcode::Divide => binary_op!(/ => Number),
				Opcode::Null => self.push_stack(Value::Null),
				Opcode::True => self.push_stack(Value::Bool(true)),
				Opcode::False => self.push_stack(Value::Bool(false)),
				Opcode::Not => {
					let input = self.pop_stack();
					if let Value::Bool(x) = input {
						self.push_stack(Value::Bool(!x))
					} else {
						runtime_error!(self, "Operand must be a boolean");
					}
				}
				Opcode::Equal => {
					let b = self.pop_stack();
					let a = self.pop_stack();
					self.push_stack(Value::Bool(a == b));
				}
				Opcode::Greater => binary_op!(> => Bool),
				Opcode::Less => binary_op!(< => Bool),
				Opcode::Print => {
					info!("{:?}", self.pop_stack());
				}
			}
		}
	}
}
