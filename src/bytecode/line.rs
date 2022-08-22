use core::fmt::{Debug, Display};

/// The line number that the instruction comes from in the user's source code.
/// Used for printing error information to the user.
#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Line {
	pub line: u16,
	pub col: u16,
}

impl Line {
	/// Constructs a new line reference.
	pub fn new(line: u16, col: u16) -> Self {
		Self { line, col }
	}
	/// Advances the line number if the char is relevant
	pub fn advance(&mut self, c: char) {
		if c == '\n' {
			self.line += 1;
			self.col = 1;
		} else {
			self.col += 1;
		}
	}
}

impl Debug for Line {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		write!(f, "{}:{}", self.line, self.col)
	}
}
impl Display for Line {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		write!(f, "{}:{}", self.line, self.col)
	}
}
