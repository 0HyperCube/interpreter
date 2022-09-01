/// Some helpful things that can be used through `use crate::prelude::*;`
pub(crate) mod prelude {
	pub use super::compiler::{scanner::*, *};
	pub use super::heap::*;
	pub use super::logger::init_logger;
	pub use super::vm::Runtime;
	pub use super::{chunk::*, errors::*, line::Line, opcode::*};
}
#[macro_use]
mod chunk;
mod compiler;
mod errors;
mod heap;
mod line;
mod logger;
mod opcode;
mod vm;

use prelude::*;

pub fn interpret(source: &str) -> Result<(), InterpretError> {
	trace!("Starting bytecode {source}");
	let chunk = Chunk::new();
	let mut runtime = Runtime::new(&chunk);
	let mut chunk = Chunk::new();
	if !Parser::compile(source, &mut chunk) {
		trace!("Compile error");
		return Err(InterpretError::CompileError);
	}
	trace!("Starting runtime chunk {:?}", chunk);
	runtime.reset(&chunk);
	runtime.interpret()?;
	trace!("Runtime ok");

	Ok(())
}

/// Reads a line of user input for the REPL
fn read_line() -> String {
	use std::io::{stdin, stdout, Write};
	let mut command = String::new();
	print!("📡 ");
	let _ = stdout().flush();
	stdin().read_line(&mut command).expect("Did not enter a correct string");
	if let Some('\n') = command.chars().next_back() {
		command.pop();
	}
	if let Some('\r') = command.chars().next_back() {
		command.pop();
	}

	command
}

/// Starts the REPL - the read evaluate print loop - for interactive testing
pub fn repl() {
	loop {
		let command = read_line();
		if command.is_empty() {
			break;
		}
		let _ = interpret(&command);
	}
}

/// Loads a file by path and runs it
pub fn run_file(path: &str) {
	let file = match std::fs::read_to_string(path) {
		Ok(file) => file,
		Err(e) => {
			error!("Error reading file: {e:?}");
			std::process::exit(74);
		}
	};
	if let Err(e) = interpret(&file) {
		match e {
			InterpretError::CompileError => std::process::exit(65),
			InterpretError::InterpretError => std::process::exit(70),
		}
	}
}
