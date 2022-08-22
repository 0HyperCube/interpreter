/// The type of error that the interpreter has found, either a compile error or an interpret error.
#[derive(Debug)]
pub enum InterpretError {
	CompileError,
	InterpretError,
}
