mod parse_rules;
mod precedence;
pub mod scanner;

use std::{cell::Ref, str::FromStr};

use crate::bytecode::prelude::*;
use parse_rules::*;
use precedence::Precedence;
pub struct Local<'source> {
	ident: Token<'source>,
	depth: usize,
}
#[derive(Default)]
pub struct Compiler<'source> {
	locals: Vec<Local<'source>>,
	depth: usize,
}

/// A simple Pratt parser that walks over the source code and output bytecode in a single pass
pub struct Parser<'a, 'source> {
	scanner: Scanner<'source>,
	current: Option<Token<'source>>,
	previous: Option<Token<'source>>,
	error: bool,
	panic: bool,
	compiling_chunk: &'a mut Chunk,
	compiler: Compiler<'source>,
}
impl<'a, 'source> Parser<'a, 'source> {
	/// Construct a new parser from the source and the target chunk
	fn new(source: &'source str, chunk: &'a mut Chunk) -> Self {
		Self {
			scanner: Scanner::new(source),
			current: None,
			previous: None,
			error: false,
			panic: false,
			compiling_chunk: chunk,
			compiler: Compiler::default(),
		}
	}
	/// Does current match the token?
	fn check(&self, token_type: TokenType) -> bool {
		self.current.as_ref().filter(|token| token.token_type == token_type).is_some()
	}
	/// Does current match the token? If so then advance.
	fn matches(&mut self, token_type: TokenType) -> bool {
		if self.check(token_type) {
			self.advance();
			true
		} else {
			false
		}
	}
	/// Are we at the end?
	fn at_end(&self) -> bool {
		self.current.as_ref().filter(|token| token.token_type != TokenType::End).is_none()
	}
	/// Create an error at the specified token
	#[track_caller]
	fn error_at(&self, token: &Token, message: &str) {
		if self.panic {
			return;
		}

		let location = std::panic::Location::caller();

		let record = log::Record::builder()
			.args(format_args!("Line"))
			.level(log::Level::Error)
			.file(Some(location.file()))
			.line(Some(location.line()))
			.target("nonew")
			.build();
		log::logger().log(&record);

		print!(" {}", token.line);
		match token.token_type {
			TokenType::Error => {}
			TokenType::End => print!(" at end"),
			_ => print!(" at '{}'", token.contents),
		}
		println!(": {}", message);
	}
	/// Create an error at the current token
	#[track_caller]
	fn error_at_current(&mut self, message: &str) {
		if let Some(token) = &self.current {
			self.error_at(token, message);
			self.error = true;
			self.panic = true;
		}
	}
	/// Create an error at the previous token (most errors)
	#[track_caller]
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
		trace!("Current {:?}", self.current);
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
			let id = self.compiling_chunk.make_constant(value);
			self.compiling_chunk.push_constant(id, token.line, Opcode::Constant, Opcode::LongConstant)
		}
	}
	/// Make the identifier into a constant
	fn emit_string(&mut self, value: String) {
		if let Some(token) = &self.previous {
			let id = self.compiling_chunk.make_string(value);
			self.compiling_chunk.push_constant(id, token.line, Opcode::Constant, Opcode::LongConstant)
		}
	}
	/// Attempt to consume a token, creating an error on failiure and advancing on success
	#[track_caller]
	fn consume(&mut self, target: TokenType, message: &'a str) {
		if self.current.as_ref().filter(|token| token.token_type == target).is_some() {
			self.advance();
		} else {
			self.error_at_current(message);
		}
	}
	/// Parses a string literal
	fn string(&mut self, _can_assign: bool) {
		if let Some(token) = &self.previous {
			self.emit_string(token.contents[1..(token.contents.len() - 1)].to_string());
		}
	}
	/// Parses a variable identifer
	fn variable(&mut self, can_assign: bool) {
		if let Some(token) = self.previous.clone() {
			self.named_variable(&token, can_assign);
		}
	}
	pub fn named_variable(&mut self, name: &Token<'source>, can_assign: bool) {
		let local = self.resolve_local(name);
		let index = local.unwrap_or_else(|| self.compiling_chunk.make_string(name.contents.to_string()));

		if can_assign && self.matches(TokenType::Equals) {
			self.expression();
			let [short, long] = if local.is_some() {
				[Opcode::SetLocal, Opcode::SetLongLocal]
			} else {
				[Opcode::SetGlobal, Opcode::SetLongGlobal]
			};
			self.compiling_chunk.push_constant(index, name.line, short, long);
		} else {
			let [short, long] = if local.is_some() {
				[Opcode::GetLocal, Opcode::GetLongLocal]
			} else {
				[Opcode::GetGlobalVariable, Opcode::GetLongGlobalVariable]
			};
			self.compiling_chunk.push_constant(index, name.line, short, long);
		}
	}

	fn resolve_local(&mut self, name: &Token<'source>) -> Option<usize> {
		self.compiler
			.locals
			.iter()
			.enumerate()
			.rev()
			.find(|(_, local)| local.ident.contents == name.contents)
			.map(|(index, _)| index)
	}

	/// Parses a number with `str::parse`
	fn number(&mut self, _can_assign: bool) {
		if let Some(token) = &self.previous {
			self.emit_constant(Value::Number(FromStr::from_str(&token.contents.chars().filter(|&c| c != '_').collect::<String>()).unwrap()));
		}
	}
	/// Parses a grouping `(5+5)`
	fn grouping(&mut self, _can_assign: bool) {
		self.expression();
		self.consume(TokenType::RightParen, "Expected closing ')'");
	}
	/// Parses a unary expression like `-5`
	fn unary(&mut self, _can_assign: bool) {
		if let Some(token) = &self.previous {
			let token_type = token.token_type;
			self.parse_precedence(Precedence::Unary);
			match token_type {
				TokenType::Minus => self.emit_byte(Opcode::Negate),
				TokenType::Escamation => self.emit_byte(Opcode::Not),
				_ => unreachable!(),
			}
		}
	}
	/// Parses a binary expression like `5-5`
	fn binary(&mut self, _can_assign: bool) {
		if let Some(token) = &self.previous {
			let operator = token.token_type;
			let rule = get_rule(operator).precedence;
			self.parse_precedence(rule.next());
			match operator {
				TokenType::Plus => self.emit_byte(Opcode::Add),
				TokenType::Minus => self.emit_byte(Opcode::Subtract),
				TokenType::Star => self.emit_byte(Opcode::Multiply),
				TokenType::Percentage => self.emit_byte(Opcode::Modolo),
				TokenType::Slash => self.emit_byte(Opcode::Divide),
				TokenType::EqualsEquals => self.emit_byte(Opcode::Equal),
				TokenType::Greater => self.emit_byte(Opcode::Greater),
				TokenType::GreaterEqual => self.emit_bytes(Opcode::Less, Opcode::Not),
				TokenType::Less => self.emit_byte(Opcode::Less),
				TokenType::LessEqual => self.emit_bytes(Opcode::Greater, Opcode::Not),
				_ => unreachable!(),
			}
		}
	}

	/// Parses a short circuit and
	fn and(&mut self, _can_assign: bool) {
		let jump_start = self.emit_jump(Opcode::JumpIfFalse);
		self.emit_byte(Opcode::Pop);
		self.parse_precedence(Precedence::And);
		self.patch_jump(jump_start);
	}
	/// Parses a short circuit or
	fn or(&mut self, _can_assign: bool) {
		let jump_start = self.emit_jump(Opcode::JumpIfFalse);
		let jump_end = self.emit_jump(Opcode::Jump);
		self.patch_jump(jump_start);
		self.emit_byte(Opcode::Pop);
		self.parse_precedence(Precedence::Or);
		self.patch_jump(jump_end);
	}
	/// Parses literal like `true`, `false` or `null`
	fn literal(&mut self, _can_assign: bool) {
		if let Some(token) = &self.previous {
			match token.token_type {
				TokenType::True => self.emit_byte(Opcode::True),
				TokenType::False => self.emit_byte(Opcode::False),
				TokenType::Null => self.emit_byte(Opcode::Null),
				_ => unreachable!("{:?}", token.token_type),
			}
		}
	}
	/// Parses an expression using a specific [`Precedence`].
	fn parse_precedence(&mut self, precedence: Precedence) {
		self.advance();
		let prefix = self.previous.as_ref().map_or(None, |token| get_rule(token.token_type).prefix);
		let can_assign = precedence as u8 <= Precedence::Assignment as u8;
		if let Some(prefix) = prefix {
			prefix(self, can_assign);
		} else {
			self.error_at_previous("Expected expression")
		}

		while precedence as u8 <= get_rule(self.current.as_ref().unwrap().token_type).precedence as u8 {
			self.advance();
			let infix = self.previous.as_ref().map_or(None, |token| get_rule(token.token_type).infix);
			if let Some(infix) = infix {
				infix(self, can_assign);
			} else {
				self.error_at_previous("Expected expression")
			}
		}

		if can_assign && self.check(TokenType::Equals) {
			warn!("curr {:?}", self.current);
			self.error_at_current("Invalid assignment target.");
		}
	}
	/// Parses with the [`Precedence::Assignment`] precedence
	fn expression(&mut self) {
		self.parse_precedence(Precedence::Assignment);
	}

	fn print_statement(&mut self) {
		self.consume(TokenType::LeftParen, "Print statements must have a '(' after the print keyword");
		self.expression();
		self.consume(TokenType::RightParen, "Print statements must end with a ')'");
		self.consume(TokenType::Semicolon, "Print statements must end with a ';'");
		self.emit_byte(Opcode::Print);
	}

	/// A statent that is just an expression e.g. `5+3;` or `foo(bar);`
	fn expression_statement(&mut self) {
		self.expression();
		self.consume(TokenType::Semicolon, "Statements must end with a ';'");
		self.emit_byte(Opcode::Pop);
	}

	/// Parse a statement (expression, for, if, pring, return, while or block)
	fn statement(&mut self) {
		if self.matches(TokenType::Print) {
			self.print_statement();
		} else if self.matches(TokenType::If) {
			self.if_statement();
		} else if self.matches(TokenType::While) {
			self.while_statement();
		} else if self.matches(TokenType::LeftBrace) {
			self.begin_scope();
			self.block();
			self.end_scope();
		} else {
			self.expression_statement();
		}
	}

	fn block(&mut self) {
		while !self.check(TokenType::RightBrace) && !self.check(TokenType::End) {
			self.declaration();
		}
		self.consume(TokenType::RightBrace, "Blocks should end with '}'.");
	}

	fn if_statement(&mut self) {
		self.expression();

		let then_jump = self.emit_jump(Opcode::JumpIfFalse);
		self.emit_byte(Opcode::Pop);

		self.consume(TokenType::LeftBrace, "If statements must contain a block");
		self.begin_scope();
		self.block();
		self.end_scope();

		let else_jump = self.emit_jump(Opcode::Jump);

		self.patch_jump(then_jump);
		self.emit_byte(Opcode::Pop);

		if self.matches(TokenType::Else) {
			self.consume(TokenType::LeftBrace, "If statements must contain a block");
			self.begin_scope();
			self.block();
			self.end_scope();
		}

		self.patch_jump(else_jump);
	}

	fn while_statement(&mut self) {
		let loop_start = self.compiling_chunk.len();
		self.expression();
		let exit = self.emit_jump(Opcode::JumpIfFalse);
		self.emit_byte(Opcode::Pop);

		self.consume(TokenType::LeftBrace, "While statements must contain a block");
		self.begin_scope();
		self.block();
		self.end_scope();

		self.jump_back(loop_start);

		self.patch_jump(exit);
	}

	/// The jump location is not specified and will be added later
	fn emit_jump(&mut self, opcode: Opcode) -> usize {
		self.emit_byte(opcode);
		self.emit_bytes(u8::MAX, u8::MAX);
		self.compiling_chunk.len() - 2
	}

	fn patch_jump(&mut self, start: usize) {
		let jump = self.compiling_chunk.len() - start - 2;
		if jump > u16::MAX as usize {
			self.error_at_current("Jump too big");
			return;
		}
		self.compiling_chunk.code[start] = (jump >> 8) as u8;
		self.compiling_chunk.code[start + 1] = jump as u8;
	}

	fn jump_back(&mut self, to: usize) {
		let jump = self.compiling_chunk.len() + 3 - to;
		if jump > u16::MAX as usize {
			self.error_at_current("Jump too big");
			return;
		}
		self.emit_byte(Opcode::JumpBack);
		self.emit_bytes((jump >> 8) as u8, jump as u8);
	}

	/// Add the jump length to a previous jump instruction

	fn begin_scope(&mut self) {
		self.compiler.depth += 1;
	}
	fn end_scope(&mut self) {
		self.compiler.depth -= 1;
		while let Some(last) = self.compiler.locals.last().filter(|last| last.depth > self.compiler.depth) {
			self.emit_byte(Opcode::Pop);
			self.compiler.locals.pop();
		}
	}

	/// After an error skip tokens until we find a new statement
	fn synchronise_error(&mut self) {
		self.panic = false;
		while !self.at_end() {
			if self.previous.as_ref().filter(|previous| previous.token_type == TokenType::Semicolon).is_some() {
				break;
			}
			if matches!(
				self.current,
				Some(Token {
					token_type: TokenType::Fn | TokenType::Let | TokenType::For | TokenType::If | TokenType::Print | TokenType::Return, // | TokenType::While
					..
				})
			) {
				return;
			}
			self.advance();
		}
	}

	fn declare_variable(&mut self, token: Token<'source>) {
		if self.compiler.depth == 0 {
			return;
		}
		self.compiler.locals.push(Local {
			ident: token,
			depth: self.compiler.depth,
		})
	}

	fn parse_variable(&mut self, message: &'static str) -> Option<(usize, Line)> {
		self.consume(TokenType::Identifier, message);

		if let Some(token) = &self.previous {
			if self.compiler.depth > 0 {
				return None;
			}

			let id = self.compiling_chunk.make_string(token.contents.to_string());
			info!("Made constant {id} {}", token.contents);
			Some((id, token.line))
		} else {
			None
		}
	}

	fn define_variable(&mut self, index: usize, line: Line) {
		if self.compiler.depth > 0 {
			return;
		}
		info!("Defining variable {index} {line}");
		self.compiling_chunk.push_constant(index, line, Opcode::DefineGlobalVariable, Opcode::DefineLongGlobalVariable)
	}

	fn variable_declaration(&mut self) {
		let global = self.parse_variable("Expected variable name.");
		let token = self.previous.clone();

		if self.matches(TokenType::Equals) {
			self.expression();
		} else {
			self.emit_byte(Opcode::Null);
		}

		self.consume(TokenType::Semicolon, "Expected ';' after variable declaration");

		if let Some((index, line)) = global {
			self.define_variable(index, line);
		} else if let Some(token) = token {
			self.declare_variable(token);
		}
	}

	/// Parse a declaration (class, function, variable or statement)
	fn declaration(&mut self) {
		if self.matches(TokenType::Let) {
			self.variable_declaration();
		} else {
			self.statement();
		}

		if self.panic {
			self.synchronise_error();
		}
	}

	/// Compiles the source into the specified chunk, returing true if successful
	pub fn compile(source: &'source str, chunk: &'a mut Chunk) -> bool {
		let mut parser = Parser::new(source, chunk);
		parser.advance();
		while parser.current.as_ref().filter(|token| token.token_type != TokenType::End).is_some() {
			parser.declaration();
		}

		parser.emit_return();
		!parser.error
	}
}
