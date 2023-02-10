use super::precedence::Precedence;
use crate::prelude::TokenType;

/// The function that will execute to parse following the specified token
pub type ParseFn<'r, 'source> = Option<fn(&mut super::Parser<'r, 'source>)>;

/// A single row in the parser table containing the prefix parse fn, the infix parse fn and the precedence
pub struct ParseRule<'r, 'source> {
	pub prefix: ParseFn<'r, 'source>,
	pub infix: ParseFn<'r, 'source>,
	pub precedence: Precedence,
}

/// Get the [ParseRule] for a specific token type
#[rustfmt::skip]
pub fn get_rule<'r,'source>(token_type: TokenType) -> ParseRule<'r,'source> {

	fn new<'r, 'source>(prefix:  ParseFn<'r,'source>, infix: ParseFn<'r,'source>, precedence: Precedence) -> ParseRule<'r,'source> {
		ParseRule { prefix, infix, precedence }
	}
	use TokenType::*;
	use super::Parser;

	match token_type {
		LeftParen        => new(Some(Parser::grouping), None,                    Precedence::None      ),
		RightParen       => new(None,                   None,                    Precedence::None      ),
		LeftBrace        => new(None,                   None,                    Precedence::None      ),
		RightBrace       => new(None,                   None,                    Precedence::None      ),
		Comma            => new(None,                   None,                    Precedence::None      ),
		Dot              => new(None,                   None,                    Precedence::None      ),
		Minus            => new(Some(Parser::unary),    Some(Parser::binary),    Precedence::Term      ),
		Plus             => new(None,                   Some(Parser::binary),    Precedence::Term      ),
		Semicolon        => new(None,                   None,                    Precedence::None      ),
		Slash            => new(None,                   Some(Parser::binary),    Precedence::Factor    ),
		Star             => new(None,                   Some(Parser::binary),    Precedence::Factor    ),
		Escamation       => new(Some(Parser::unary),    None,                    Precedence::None      ),
		EscamationEquals => new(None,                   None,                    Precedence::None      ),
		Equals           => new(None,                   None,                    Precedence::None      ),
		EqualsEquals     => new(None,                   Some(Parser::binary),    Precedence::Comparison),
		Greater          => new(None,                   Some(Parser::binary),    Precedence::Comparison),
		GreaterEqual     => new(None,                   Some(Parser::binary),    Precedence::Comparison),
		Less             => new(None,                   Some(Parser::binary),    Precedence::Comparison),
		LessEqual        => new(None,                   Some(Parser::binary),    Precedence::Comparison),
		Identifier       => new(None,                   None,                    Precedence::None      ),
		StringLiteral    => new(Some(Parser::string),   None,                    Precedence::None      ),
		NumberLiteral    => new(Some(Parser::number),   None,                    Precedence::None      ),
		And              => new(None,                   None,                    Precedence::None      ),
		Or               => new(None,                   None,                    Precedence::None      ),
		If               => new(None,                   None,                    Precedence::None      ),
		Else             => new(None,                   None,                    Precedence::None      ),
		True             => new(Some(Parser::literal),  None,                    Precedence::None      ),
		False            => new(Some(Parser::literal),  None,                    Precedence::None      ),
		For              => new(None,                   None,                    Precedence::None      ),
		While            => new(None,                   None,                    Precedence::None      ),
		Fn               => new(None,                   None,                    Precedence::None      ),
		Print            => new(None,                   None,                    Precedence::None      ),
		Return           => new(None,                   None,                    Precedence::None      ),
		Let              => new(None,                   None,                    Precedence::None      ),
		Null             => new(Some(Parser::literal),  None,                    Precedence::None      ),
		Error            => new(None,                   None,                    Precedence::None      ),
		End              => new(None,                   None,                    Precedence::None      ),
	}
}
