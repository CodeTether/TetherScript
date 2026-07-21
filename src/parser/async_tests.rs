//! Async declaration and expression parser tests.

use crate::ast::{Expr, Stmt};
use crate::lexer::Lexer;

use super::Parser;

#[test]
fn named_async_function_retains_async_identity() {
    let tokens = Lexer::new("async fn work(value) { return value }")
        .tokenize()
        .unwrap();
    let program = Parser::new(tokens).parse_program().unwrap();
    assert!(matches!(
        &program.stmts[0],
        Stmt::Let {
            name,
            value: Expr::AsyncFn { params, .. },
            ..
        } if name == "work" && params == &["value"]
    ));
}

#[test]
fn concurrency_expressions_keep_their_ast_nodes() {
    let tokens = Lexer::new("fn main() { join(spawn 1, await async fn() { 2 }()) }")
        .tokenize()
        .unwrap();
    let program = Parser::new(tokens).parse_program().unwrap();
    let Stmt::FnDecl { body, .. } = &program.stmts[0] else {
        panic!("main should parse as a declaration");
    };
    assert!(matches!(
        &body.stmts[0],
        Stmt::Expr {
            expr: Expr::Join(_),
            ..
        }
    ));
}
