//! Parse helpers shared by the infix operator tests.

use crate::ast::{BinOp, Expr, Stmt};
use crate::lexer::Lexer;
use crate::parser::Parser;

/// Parse `source` and return the operator of its first `let` initializer.
pub(super) fn let_operator(source: &str) -> BinOp {
    let Some(Stmt::Let { value, .. }) = program(source).stmts.into_iter().next() else {
        panic!("expected a let statement");
    };
    match value {
        Expr::Binary { op, .. } => op,
        other => panic!("expected a binary expression, got {other:?}"),
    }
}

/// Parse `source` and return the initializer of its `index`-th statement.
pub(super) fn let_value(source: &str, index: usize) -> Expr {
    let Some(Stmt::Let { value, .. }) = program(source).stmts.into_iter().nth(index) else {
        panic!("expected a let statement at {index}");
    };
    value
}

/// Report whether `source` lexes and parses cleanly.
pub(super) fn parses(source: &str) -> bool {
    match Lexer::new(source).tokenize() {
        Ok(tokens) => Parser::new(tokens).parse_program().is_ok(),
        Err(_) => false,
    }
}

fn program(source: &str) -> crate::ast::Program {
    let tokens = Lexer::new(source).tokenize().expect("source should lex");
    Parser::new(tokens)
        .parse_program()
        .expect("source should parse")
}
