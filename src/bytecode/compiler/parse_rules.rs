use crate::prelude::TokenType;

use super::precedence::Precedence;

/// The function that will execute to parse following the specified token
pub enum ParseFn {
	None,
	Grouping,
	Unary,
	Binary,
	Number,
}

/// A single row in the parser table containing the prefix parse fn, the infix parse fn and the precedence
pub struct ParseRule {
	pub prefix: ParseFn,
	pub infix: ParseFn,
	pub precedence: Precedence,
}

/// Get the [ParseRule] for a specific token type
#[rustfmt::skip]
pub fn get_rule(token_type: TokenType) -> ParseRule {

	const fn new(prefix: ParseFn, infix: ParseFn, precedence: Precedence) -> ParseRule {
		ParseRule { prefix, infix, precedence }
	}
	use TokenType::*;

	match token_type {
		LeftParen        => new(ParseFn::Grouping, ParseFn::None,     Precedence::None      ),
		RightParen       => new(ParseFn::None,     ParseFn::None,     Precedence::None      ),
		LeftBrace        => new(ParseFn::None,     ParseFn::None,     Precedence::None      ),
		RightBrace       => new(ParseFn::None,     ParseFn::None,     Precedence::None      ),
		Comma            => new(ParseFn::None,     ParseFn::None,     Precedence::None      ),
		Dot              => new(ParseFn::None,     ParseFn::None,     Precedence::None      ),
		Minus            => new(ParseFn::Unary,    ParseFn::Binary,   Precedence::Term      ),
		Plus             => new(ParseFn::None,     ParseFn::Binary,   Precedence::Term      ),
		Semicolon        => new(ParseFn::None,     ParseFn::None,     Precedence::None      ),
		Slash            => new(ParseFn::None,     ParseFn::Binary,   Precedence::Factor    ),
		Star             => new(ParseFn::None,     ParseFn::Binary,   Precedence::Factor    ),
		Escamation       => new(ParseFn::None,     ParseFn::None,     Precedence::None      ),
		EscamationEquals => new(ParseFn::None,     ParseFn::None,     Precedence::None      ),
		Equals           => new(ParseFn::None,     ParseFn::None,     Precedence::None      ),
		EqualsEquals     => new(ParseFn::None,     ParseFn::None,     Precedence::None      ),
		Greater          => new(ParseFn::None,     ParseFn::None,     Precedence::None      ),
		GreaterEqual     => new(ParseFn::None,     ParseFn::None,     Precedence::None      ),
		Less             => new(ParseFn::None,     ParseFn::None,     Precedence::None      ),
		LessEqual        => new(ParseFn::None,     ParseFn::None,     Precedence::None      ),
		Identifier       => new(ParseFn::None,     ParseFn::None,     Precedence::None      ),
		StringLiteral    => new(ParseFn::None,     ParseFn::None,     Precedence::None      ),
		NumberLiteral    => new(ParseFn::Number,   ParseFn::None,     Precedence::None      ),
		And              => new(ParseFn::None,     ParseFn::None,     Precedence::None      ),
		Or               => new(ParseFn::None,     ParseFn::None,     Precedence::None      ),
		If               => new(ParseFn::None,     ParseFn::None,     Precedence::None      ),
		Else             => new(ParseFn::None,     ParseFn::None,     Precedence::None      ),
		True             => new(ParseFn::None,     ParseFn::None,     Precedence::None      ),
		False            => new(ParseFn::None,     ParseFn::None,     Precedence::None      ),
		For              => new(ParseFn::None,     ParseFn::None,     Precedence::None      ),
		Fn               => new(ParseFn::None,     ParseFn::None,     Precedence::None      ),
		Return           => new(ParseFn::None,     ParseFn::None,     Precedence::None      ),
		Let              => new(ParseFn::None,     ParseFn::None,     Precedence::None      ),
		Null             => new(ParseFn::None,     ParseFn::None,     Precedence::None      ),
		Error            => new(ParseFn::None,     ParseFn::None,     Precedence::None      ),
		End              => new(ParseFn::None,     ParseFn::None,     Precedence::None      ),
	}
}
