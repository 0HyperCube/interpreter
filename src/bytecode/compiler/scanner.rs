use core::str::Chars;
use std::cell::{Ref, RefCell};

use crate::bytecode::prelude::*;

/// The type of the token
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
	// Single character
	/// (
	LeftParen,
	/// )
	RightParen,
	/// {
	LeftBrace,
	/// }
	RightBrace,
	/// ,
	Comma,
	/// .
	Dot,
	/// -
	Minus,
	/// +
	Plus,
	/// ;
	Semicolon,
	/// /
	Slash,
	/// *
	Star,

	// One or two characters
	/// !
	Escamation,
	/// !=
	EscamationEquals,
	/// =
	Equals,
	/// ==
	EqualsEquals,
	/// >
	Greater,
	/// >=
	GreaterEqual,
	/// <
	Less,
	/// <=
	LessEqual,

	// Literal
	/// bob
	Identifier,
	/// "bob"
	StringLiteral,
	/// 3.14
	NumberLiteral,

	// Keywords
	And,
	Or,
	If,
	Else,
	True,
	False,
	For,
	While,
	Fn,
	Return,
	Let,
	Null,
	Print,

	Error,
	End,
}

/// A fragment of user source code with a particular meaning
#[derive(Debug, Clone)]
pub struct Token<'a> {
	pub token_type: TokenType,
	pub contents: &'a str,
	pub line: Line,
}

/// An iter that can be peeked 2 items in advance
struct Peekable<T: Copy, I: Iterator<Item = T>> {
	iter: I,
	peek1: Option<Option<I::Item>>,
	peek2: Option<Option<I::Item>>,
}
impl<T: Copy, I: Iterator<Item = T>> Peekable<T, I> {
	/// Construct a new Peekable from the iterator specified
	pub fn new(iter: I) -> Self {
		Self { iter, peek1: None, peek2: None }
	}
	/// Advance the iterator
	pub fn next(&mut self) -> Option<I::Item> {
		let result = match self.peek1.take() {
			Some(x) => x,
			None => self.iter.next(),
		};
		self.peek1 = self.peek2.take();
		result
	}
	/// Look one ahead without advancing the iter
	pub fn peek1(&mut self) -> Option<I::Item> {
		let result = match self.peek1.take() {
			Some(x) => x,
			None => self.iter.next(),
		};
		self.peek1 = Some(result);
		result
	}
	/// Look two items ahead without advancing the iter
	pub fn peek2(&mut self) -> Option<I::Item> {
		self.peek1 = Some(match self.peek1.take() {
			Some(x) => x,
			None => self.iter.next(),
		});
		let result = match self.peek2.take() {
			Some(x) => x,
			None => self.iter.next(),
		};
		self.peek2 = Some(result);
		result
	}
}

#[test]
fn peekable() {
	let mut peek = Peekable::new(0..30);
	assert_eq!(peek.next(), Some(0));
	assert_eq!(peek.next(), Some(1));
	assert_eq!(peek.next(), Some(2));
	assert_eq!(peek.peek2(), Some(4));
	assert_eq!(peek.peek1(), Some(3));
	assert_eq!(peek.peek2(), Some(4));
	assert_eq!(peek.next(), Some(3));
	assert_eq!(peek.next(), Some(4));
	assert_eq!(peek.next(), Some(5));
}

/// The scanner which looks through the source code and generates tokens
pub struct Scanner<'a> {
	source: &'a str,
	chars: Peekable<char, Chars<'a>>,
	start: usize,
	start_line: Line,
	current: usize,
	line: Line,
	string_nesting: usize,
}

impl<'a> Scanner<'a> {
	/// Construct a new scanner with the specified source code
	pub fn new(source: &'a str) -> Self {
		Scanner {
			source,
			chars: Peekable::new(source.chars()),
			start: 0,
			start_line: Line::new(1, 1),
			current: 0,
			line: Line::new(1, 1),
			string_nesting: 0,
		}
	}
	/// Construct a new token with the specified type and the stored start and line
	fn new_token(&self, token_type: TokenType) -> Token<'a> {
		Token {
			token_type,
			contents: &self.source[self.start..self.current],
			line: self.start_line,
		}
	}
	/// Construct an error token with the specified type and the stored start and line
	fn new_error(&self, message: &'static str) -> Token<'a> {
		Token {
			token_type: TokenType::Error,
			contents: message,
			line: self.start_line,
		}
	}
	/// Check if we have reached the end of the source code
	#[inline]
	fn at_end(&self) -> bool {
		self.current >= self.source.len()
	}
	/// Advance the scanner by one character, returning it if we have not reached the end
	fn advance(&mut self) -> Option<char> {
		if let Some(c) = self.chars.next() {
			self.current += c.len_utf8();
			self.line.advance(c);
			Some(c)
		} else {
			None
		}
	}
	/// Skip whitespace including comments
	fn skip_spaces(&mut self) -> Result<(), &'static str> {
		loop {
			match self.chars.peek1() {
				Some('\t' | '\n' | '\x0C' | '\r' | ' ') => {
					self.advance();
				}
				// Comments are treated as whitespace
				Some('/') => match self.chars.peek2() {
					Some('/') => {
						while self.chars.peek1() != Some('\n') && !self.at_end() {
							self.advance();
						}
					}
					Some('*') => {
						self.start_line = self.line;
						self.advance();
						self.advance();
						while !(self.chars.peek1() == Some('*') && self.chars.peek2() == Some('/')) {
							self.advance();
							if self.at_end() {
								return Err("Unclosed multiline comment");
							}
						}
						self.advance();
						self.advance();
					}
					_ => break,
				},
				_ => break,
			}
		}
		Ok(())
	}
	/// Consume a string literal in the user's source code which is surrounded by double quotes
	fn comsume_string(&mut self) -> Token<'a> {
		while !self.matches('"') {
			self.advance();
			if self.at_end() {
				return self.new_error("Unclosed string");
			}
		}
		self.new_token(TokenType::StringLiteral)
	}
	/// Consume a number literal in the user's source code wich is a sequence of digits optionally containing a decimal point
	fn comsume_number(&mut self) -> Token<'a> {
		while self.chars.peek1().filter(|c| c.is_ascii_digit()).is_some() {
			self.advance();
		}
		if self.matches('.') && self.chars.peek2().filter(|c| c.is_ascii_digit()).is_some() {
			while self.chars.peek1().filter(|c| c.is_ascii_digit()).is_some() {
				self.advance();
			}
		}
		self.new_token(TokenType::NumberLiteral)
	}
	/// Checks if the current token is part of a keyword
	fn check_keyword(&self, start_offset: usize, val: &str, token_type: TokenType) -> TokenType {
		if val.len() == self.current - (self.start + start_offset) {
			if &self.source[self.start + start_offset..self.current] == val {
				return token_type;
			}
		}
		TokenType::Identifier
	}
	/// Consumes an identifer, checking if it is a keyword or a user identifier
	fn comsume_ident(&mut self) -> Token<'a> {
		while self.chars.peek1().filter(|c| c.is_alphanumeric()).is_some() {
			self.advance();
		}

		let token_type = match self.get_byte(self.start as isize) {
			b'a' => self.check_keyword(1, "nd", TokenType::And),
			b'o' => self.check_keyword(1, "r", TokenType::Or),
			b'i' => self.check_keyword(1, "f", TokenType::If),
			b'e' => self.check_keyword(1, "lse", TokenType::Else),
			b't' => self.check_keyword(1, "rue", TokenType::True),
			b'f' => match self.get_byte(self.start as isize + 1) {
				b'a' => self.check_keyword(2, "lse", TokenType::False),
				b'o' => self.check_keyword(2, "r", TokenType::For),
				b'n' => self.check_keyword(2, "", TokenType::Fn),
				_ => TokenType::Identifier,
			},
			b'r' => self.check_keyword(1, "eturn", TokenType::Return),
			b'l' => self.check_keyword(1, "et", TokenType::Let),
			b'n' => self.check_keyword(1, "ull", TokenType::Null),
			b'p' => self.check_keyword(1, "rint", TokenType::Print),
			_ => TokenType::Identifier,
		};
		info!("Token {:?}", token_type);
		self.new_token(token_type)
	}
	/// Get the byte at the specified position
	fn get_byte(&self, byte: isize) -> u8 {
		unsafe { *self.source.as_ptr().offset(byte) }
	}
	/// Try to consume the character specified, returning false if impossible
	fn matches(&mut self, val: char) -> bool {
		if self.chars.peek1().filter(|&c| c == val).is_some() {
			self.advance();
			true
		} else {
			false
		}
	}
	/// Parses the next token
	pub fn next(&mut self) -> Token<'a> {
		if let Err(e) = self.skip_spaces() {
			return self.new_error(e);
		}
		self.start = self.current;
		self.start_line = self.line;
		let next = match self.advance() {
			Some(c) => c,
			None => return self.new_token(TokenType::End),
		};
		match next {
			'(' => self.new_token(TokenType::LeftParen),
			')' => self.new_token(TokenType::RightParen),
			'{' => self.new_token(TokenType::LeftBrace),
			'}' => self.new_token(TokenType::RightBrace),
			',' => self.new_token(TokenType::Comma),
			'.' => self.new_token(TokenType::Dot),
			'+' => self.new_token(TokenType::Plus),
			'-' => self.new_token(TokenType::Minus),
			';' => self.new_token(TokenType::Semicolon),
			'/' => self.new_token(TokenType::Slash),
			'*' => self.new_token(TokenType::Star),

			'!' => {
				let token_type = if self.matches('=') { TokenType::EscamationEquals } else { TokenType::Escamation };
				self.new_token(token_type)
			}
			'=' => {
				let token_type = if self.matches('=') { TokenType::EqualsEquals } else { TokenType::Equals };
				self.new_token(token_type)
			}
			'>' => {
				let token_type = if self.matches('=') { TokenType::GreaterEqual } else { TokenType::Greater };
				self.new_token(token_type)
			}
			'<' => {
				let token_type = if self.matches('=') { TokenType::LessEqual } else { TokenType::Less };
				self.new_token(token_type)
			}

			'"' => self.comsume_string(),
			_ if next.is_ascii_digit() => self.comsume_number(),
			_ if next.is_alphabetic() => self.comsume_ident(),

			_ => self.new_error("Unknown character"),
		}
	}
}

#[test]
fn scanner() {
	init_logger();
	let mut scanner = Scanner::new(
		r#"
(=>=)/*/po*/{}//f
"hello"

blobby
fnc
fn
老 fn 🌏
"#,
	);
	loop {
		let token = scanner.next();
		println!("{token:?}");
		if token.token_type == TokenType::End {
			break;
		}
	}
}
