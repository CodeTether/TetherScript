//! Parse one module file with path-qualified diagnostics.

use std::fs;
use std::path::Path;

use crate::ast::Program;
use crate::lexer::Lexer;
use crate::parser::Parser;

pub(super) fn file(path: &Path) -> Result<Program, String> {
    let source = fs::read_to_string(path)
        .map_err(|error| format!("can't read module {}: {error}", path.display()))?;
    let tokens = Lexer::new(&source).tokenize().map_err(|error| {
        format!(
            "{}:{}:{}: lex error: {}",
            path.display(),
            error.line,
            error.col,
            error.msg
        )
    })?;
    Parser::new(tokens).parse_program().map_err(|error| {
        format!(
            "{}:{}:{}: parse error: {}",
            path.display(),
            error.line,
            error.col,
            error.msg
        )
    })
}
