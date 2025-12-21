//! Coding rules (OneStatementPerLine, MultipleVariableDeclarations, etc.)

mod multiple_variable_declarations;
mod one_statement_per_line;
mod simplify_boolean_return;

pub use multiple_variable_declarations::MultipleVariableDeclarations;
pub use one_statement_per_line::OneStatementPerLine;
pub use simplify_boolean_return::SimplifyBooleanReturn;
