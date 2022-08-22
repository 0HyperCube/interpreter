use crate::bytecode::prelude::*;

/// The interpeter's runtime, containing the current [Chunk], a pointer to the next instruction and the stack
pub struct Runtime<'a> {
	/// The [`Chunk`] that is being interpreted
	chunk: &'a Chunk,
	/// The instruction pointer, pointing to the next instruction
	ip: *const u8,
	/// The stack of values that can be pushed to and popped from
	stack: Vec<Value>,
	/// Pointer to the top of the stack
	stack_top: *mut Value,
}

impl<'a> Runtime<'a> {
	/// Construct a new runtime with the specified [Chunk]
	pub fn new(chunk: &'a Chunk) -> Self {
		let mut stack = Vec::with_capacity(5);
		Self {
			chunk,
			ip: chunk.as_ptr(),
			stack_top: stack.as_mut_ptr(),
			stack,
		}
	}

	pub fn reset(&mut self, chunk: &'a Chunk) {
		self.chunk = chunk;
		self.ip = chunk.as_ptr();
		self.stack.clear();
		self.stack_top = self.stack.as_mut_ptr();
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
	pub fn short_constant(&mut self) -> &'a Value {
		self.chunk.constant(self.read_byte() as usize)
	}

	/// Read a long constant from the [Chunk].
	#[inline]
	pub fn long_constant(&mut self) -> &'a Value {
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
	pub fn push_stack(&mut self, value: Value) {
		unsafe {
			*self.stack_top = value;

			// Grow the vec if too small
			if self.stack.capacity() < self.stack.as_ptr().offset_from(self.stack_top) as usize {
				self.stack.reserve(1);
			}
			self.stack_top = self.stack_top.offset(1);
		}
	}
	// Pops an item from the top of the stack, returning it
	#[inline]
	pub fn pop_stack(&mut self) -> Value {
		unsafe {
			self.stack_top = self.stack_top.offset(-1);
			*self.stack_top
		}
	}

	/// Interprets the [Chunk], matching each opcode instruction.
	pub fn interpret(&mut self) -> Result<(), InterpretError> {
		trace!("Interpreting chunk");
		loop {
			#[cfg(feature = "trace_execution")]
			{
				let mut current = self.stack.as_ptr();

				if current != self.stack_top {
					trace!(target: "Stack", "");
					while current != self.stack_top {
						unsafe {
							print!("[ {} ]", *current);
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
				($op:tt) => {
					{
						let b = self.pop_stack();
						let a = self.pop_stack();
						self.push_stack(a $op b);
					}
				};
			}

			match opcode {
				Opcode::Unknown => warn!("Unknown opcode"),

				Opcode::Constant => {
					let constant = *self.short_constant();
					self.push_stack(constant);
				}
				Opcode::LongConstant => {
					let constant = *self.long_constant();
					self.push_stack(constant);
				}
				Opcode::Return => return Ok(()),
				Opcode::Negate => {
					let input = self.pop_stack();
					self.push_stack(-input);
				}
				Opcode::Add => binary_op!(+),
				Opcode::Subtract => binary_op!(-),
				Opcode::Multiply => binary_op!(*),
				Opcode::Divide => binary_op!(/),
			}
		}
	}
}
