/// Expressions will chew through all higher [Precedence] items.
#[derive(Clone, Copy)]
pub enum Precedence {
	None,
	Assignment,
	Or,
	And,
	Equality,
	Comparison,
	/// Addition and subtraction
	Term,
	/// Multiplication and division
	Factor,
	Unary,
	/// Function call
	Call,
	Primary,
}
impl Precedence {
	/// Adds one to the precedence
	pub fn next(&self) -> Self {
		match self {
			Precedence::None => Precedence::Assignment,
			Precedence::Assignment => Precedence::Or,
			Precedence::Or => Precedence::And,
			Precedence::And => Precedence::Equality,
			Precedence::Equality => Precedence::Comparison,
			Precedence::Comparison => Precedence::Term,
			Precedence::Term => Precedence::Factor,
			Precedence::Factor => Precedence::Unary,
			Precedence::Unary => Precedence::Call,
			Precedence::Call => Precedence::Primary,
			Precedence::Primary => Precedence::Primary,
		}
	}
}
