use crate::{ast::Program, lexer::Lexer, parser::Parser, token::Spanned};

pub(super) fn tokens(source: &str) -> Result<Vec<Spanned>, String> {
    Lexer::new(source)
        .tokenize()
        .map_err(|error| format!("lex error at {}:{}: {}", error.line, error.col, error.msg))
}

pub(super) fn program(tokens: Vec<Spanned>) -> Result<Program, String> {
    Parser::new(tokens)
        .parse_program()
        .map_err(|error| format!("parse error at {}:{}: {}", error.line, error.col, error.msg))
}
