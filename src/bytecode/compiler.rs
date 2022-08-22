mod parse_rules;
mod precedence;
pub mod scanner;

use crate::bytecode::prelude::*;
use parse_rules::*;
use precedence::Precedence;

/// A simple Pratt parser that walks over the source code and output bytecode in a single pass
pub struct Parser<'a> {
	scanner: Scanner<'a>,
	current: Option<Token<'a>>,
	previous: Option<Token<'a>>,
	error: bool,
	panic: bool,
	compiling_chunk: &'a mut Chunk,
}
impl<'a> Parser<'a> {
	/// Construct a new parser from the source and the target chunk
	fn new(source: &'a str, chunk: &'a mut Chunk) -> Self {
		Self {
			scanner: Scanner::new(source),
			current: None,
			previous: None,
			error: false,
			panic: false,
			compiling_chunk: chunk,
		}
	}
	/// Create an error at the specified token
	fn error_at(&self, token: &Token, message: &str) {
		if self.panic {
			return;
		}

		error!(target: "Source Error", "Line {}", token.line);
		match token.token_type {
			TokenType::Error => {}
			TokenType::End => print!(" at end"),
			_ => print!(" at '{}'", token.contents),
		}
		println!(": {}", message);
	}
	/// Create an error at the current token
	fn error_at_current(&mut self, message: &str) {
		if let Some(token) = &self.current {
			self.error_at(token, message);
			self.error = true;
			self.panic = true;
		}
	}
	/// Create an error at the previous token (most errors)
	fn error_at_previous(&mut self, message: &str) {
		if let Some(token) = &self.previous {
			self.error_at(token, message);
			self.error = true;
			self.panic = true;
		}
	}
	/// Advance to the next token, skipping any errors
	fn advance(&mut self) {
		self.previous = self.current.take();

		loop {
			let new = self.scanner.next();
			if new.token_type != TokenType::Error {
				self.current = Some(new);
				break;
			} else {
				let msg = new.contents;
				self.current = Some(new);
				self.error_at_current(msg)
			}
		}
	}
	/// Emits a byte with the line number of the previous token
	fn emit_byte(&mut self, byte: impl Into<u8>) {
		if let Some(token) = &self.previous {
			self.compiling_chunk.push(byte, token.line);
		}
	}
	/// Emits 2 bytes with the line number of the previous token
	fn emit_bytes(&mut self, byte1: impl Into<u8>, byte2: impl Into<u8>) {
		if let Some(token) = &self.previous {
			self.compiling_chunk.push(byte1, token.line);
			self.compiling_chunk.push(byte2, token.line);
		}
	}
	/// Emits a return, tracing the chunk if debugging is enabled
	fn emit_return(&mut self) {
		if let Some(token) = &self.previous {
			self.compiling_chunk.push(Opcode::Return, token.line);
		}
		#[cfg(feature = "trace_execution")]
		disassemble!(chunk = &self.compiling_chunk, name = "code");
	}
	/// Emit a constant at the last token
	fn emit_constant(&mut self, value: Value) {
		if let Some(token) = &self.previous {
			self.compiling_chunk.push_constant(value, token.line)
		}
	}
	/// Attempt to consume a token, creating an error on failiure and advancing on success
	fn consume(&mut self, target: TokenType, message: &'a str) {
		if self.current.as_ref().filter(|token| token.token_type == target).is_some() {
			self.advance();
		} else {
			self.error_at_current(message);
		}
	}
	/// Parses a number with `str::parse`
	fn number(&mut self) {
		if let Some(token) = &self.previous {
			self.emit_constant(token.contents.parse().unwrap());
		}
	}
	/// Parses a grouping `(5+5)`
	fn grouping(&mut self) {
		self.expression();
		self.consume(TokenType::RightParen, "Expected closing ')'");
	}
	/// Parses a unary expression like `-5`
	fn unary(&mut self) {
		if let Some(token) = &self.previous {
			let token_type = token.token_type;
			self.parse_precedence(Precedence::Unary);
			match token_type {
				TokenType::Minus => self.emit_byte(Opcode::Negate),
				_ => unreachable!(),
			}
		}
	}
	/// Parses a binary expression like `5-5`
	fn binary(&mut self) {
		if let Some(token) = &self.previous {
			let operator = token.token_type;
			let rule = get_rule(operator).precedence.next();
			self.parse_precedence(rule.next());
			match operator {
				TokenType::Plus => self.emit_byte(Opcode::Add),
				TokenType::Minus => self.emit_byte(Opcode::Subtract),
				TokenType::Star => self.emit_byte(Opcode::Multiply),
				TokenType::Slash => self.emit_byte(Opcode::Divide),
				_ => unreachable!(),
			}
		}
	}
	/// Runs the specified [`ParseFn`]
	#[inline]
	fn parse_fn(&mut self, parse_fn: ParseFn) {
		match parse_fn {
			ParseFn::None => self.error_at_previous("Expected expression"),
			ParseFn::Grouping => self.grouping(),
			ParseFn::Unary => self.unary(),
			ParseFn::Binary => self.binary(),
			ParseFn::Number => self.number(),
		}
	}
	/// Parses an expression using a specific [`Precedence`].
	fn parse_precedence(&mut self, precedence: Precedence) {
		self.advance();
		let prefix = self.previous.as_ref().map_or(ParseFn::None, |token| get_rule(token.token_type).prefix);
		self.parse_fn(prefix);

		while precedence as u8 <= get_rule(self.current.as_ref().unwrap().token_type).precedence as u8 {
			self.advance();
			let infix = self.previous.as_ref().map_or(ParseFn::None, |token| get_rule(token.token_type).infix);
			self.parse_fn(infix);
		}
	}
	/// Parses with the [`Precedence::Assignment`] precedence
	fn expression(&mut self) {
		self.parse_precedence(Precedence::Assignment);
	}

	/// Compiles the source into the specified chunk, returing true if successful
	pub fn compile(source: &'a str, chunk: &'a mut Chunk) -> bool {
		let mut parser = Parser::new(source, chunk);
		parser.advance();
		parser.expression();
		parser.emit_return();
		!parser.error
	}
}
