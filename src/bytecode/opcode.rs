use crate::bytecode::prelude::*;

/// A simple macro for converting the opcode to and from a specified intager type
macro_rules! opcode {
	($int:ty, $(#[$macros:meta])* $vis:vis enum $name:ident { $($index:literal => $value:ident),* $(,)? }) => {
		$(#[$macros])* #[repr($int)]
		$vis enum $name { $($value = $index,)* Unknown }

		impl From<$int> for $name{
			fn from(origin: $int) -> Self{
				match origin{
					$($index => Self::$value,)*
					_ => Self::Unknown,
				}
			}
		}
		impl From<$name> for $int {
			fn from(origin: $name) -> Self { origin as $int }
		}
	};
}

opcode! {
	u8,

	/// The operation code, defining an operation in the bytecode.
	#[derive(Debug, PartialEq, Eq)]
	pub enum Opcode {
		0 => Return,

		1 => Constant,
		2 => LongConstant,

		3 => Negate,

		4 => Add,
		5 => Subtract,
		6 => Multiply,
		7 => Divide,


		8 => Null,
		9 => True,
		10 => False,

		11 => Not,
		12 => Equal,
		13 => Greater,
		14 => Less,

		15 => Print,
		16 => Pop,

		17 => DefineGlobalVariable,
		18 => DefineLongGlobalVariable,
		19 => GetGlobalVariable,
		20 => GetLongGlobalVariable,
		21 => SetGlobal,
		22 => SetLongGlobal,
		23 => GetLocal,
		24 => GetLongLocal,
		25 => SetLocal,
		26 => SetLongLocal,

		27 => Jump,
		28 => JumpIfFalse,
		29=> JumpBack,

		30 => Modolo,
	}
}

/// Disassembles an instruction, printing out information relevant for debugging and returning the new offset.
#[cfg(feature = "trace_execution")]
pub fn disassemble_instruction(chunk: &Chunk, offset: usize) -> usize {
	/// Disassembles a simple instruction of one byte.
	fn simple_instruction(opcode: Opcode, offset: usize) -> usize {
		println!("{opcode:?}");
		offset + 1
	}

	/// Disassembles the short constant instruction
	fn constant_instruction(chunk: &Chunk, opcode: Opcode, offset: usize) -> usize {
		let constant_idx = chunk[offset + 1];
		let constant = chunk.constant(constant_idx as usize);
		println!("{:<16} {constant_idx} {constant:?}", format!("{:?}", opcode));

		offset + 2
	}

	/// Disassembles the long constant instruction
	fn long_constant_instruction(chunk: &Chunk, opcode: Opcode, offset: usize) -> usize {
		let mut constant_idx = 0;
		for i in 0..3 {
			constant_idx <<= 8;
			constant_idx ^= chunk[offset + i + 1] as usize;
		}
		let constant = chunk.constant(constant_idx);
		println!("{:<16} {constant_idx} {constant:?}", format!("{:?}", opcode));

		offset + 4
	}

	/// Disassembles the long
	fn value_instruction(chunk: &Chunk, opcode: Opcode, offset: usize, length: usize) -> usize {
		let mut value = 0;
		for i in 0..length {
			value <<= 8;
			value ^= chunk[offset + i + 1] as usize;
		}
		println!("{:<16} {value}", format!("{:?}", opcode));

		offset + 1 + length
	}

	// Log the byte number
	trace!(target: "Disassembly", "{:0>4} ", offset);

	let line = chunk.lines[offset];
	// Log the line number or "|" if it is the same as the last instruction
	if offset != 0 && chunk.lines[offset - 1] == line {
		print!("     | ");
	} else {
		print!("{:>6} ", line.to_string());
	}

	let opcode_id = chunk[offset];
	let opcode = opcode_id.into();
	// Log the rest of the instructionbased on the opcode
	match opcode {
		Opcode::Unknown => {
			warn!("Unknown instruction {opcode_id}");
			offset + 1
		}

		Opcode::Constant | Opcode::DefineGlobalVariable | Opcode::GetGlobalVariable | Opcode::SetGlobal => constant_instruction(chunk, opcode, offset),
		Opcode::LongConstant | Opcode::DefineLongGlobalVariable | Opcode::GetLongGlobalVariable | Opcode::SetLongGlobal => long_constant_instruction(chunk, opcode, offset),

		Opcode::GetLocal | Opcode::SetLocal => value_instruction(chunk, opcode, offset, 1),
		Opcode::GetLongLocal | Opcode::SetLongLocal => value_instruction(chunk, opcode, offset, 3),
		Opcode::Jump | Opcode::JumpIfFalse | Opcode::JumpBack => value_instruction(chunk, opcode, offset, 2),

		_ => simple_instruction(opcode, offset),
	}
}

#[test]
fn opcode() {
	init_logger();
	info!("{}", core::mem::size_of::<Opcode>());
}
