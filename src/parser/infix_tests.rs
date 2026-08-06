//! Tests for infix-position handling, especially the `&` diagnostic.
//!
//! Regression lock for the silent-wrong-answer bug where `let a = 12 & 10;`
//! parsed as `let a = 12` plus a discarded `&10` borrow and printed `12`.

use crate::lexer::Lexer;
use crate::parser::Parser;

fn parse_err(src: &str) -> String {
    let tokens = Lexer::new(src).tokenize().expect("source should lex");
    let err = Parser::new(tokens)
        .parse_program()
        .expect_err("source should not parse");
    err.msg
}

fn parses(src: &str) -> bool {
    let Ok(tokens) = Lexer::new(src).tokenize() else {
        return false;
    };
    Parser::new(tokens).parse_program().is_ok()
}

#[test]
fn integer_amp_integer_is_rejected() {
    let msg = parse_err("let a = 12 & 10;");
    assert!(msg.contains("`&` is not a binary operator"), "got: {msg}");
    assert!(msg.contains("bitwise AND"), "got: {msg}");
}

#[test]
fn amp_error_suggests_logical_and_and_borrow() {
    let msg = parse_err("let a = 12 & 10;");
    assert!(msg.contains("`&&`"), "got: {msg}");
    assert!(msg.contains("borrow"), "got: {msg}");
}

#[test]
fn amp_after_identifier_is_rejected() {
    let msg = parse_err("let x = 1;\nlet y = x & x;");
    assert!(msg.contains("`&` is not a binary operator"), "got: {msg}");
}

#[test]
fn prefix_borrow_still_parses() {
    assert!(parses("let x = 1;\nlet y = &x;"));
    assert!(parses("let mut m = 1;\nlet r = &mut m;"));
}

#[test]
fn logical_and_still_parses() {
    assert!(parses("let a = true && false;"));
}

#[test]
fn borrow_argument_still_parses() {
    assert!(parses("fn f(v) { v }\nlet x = 1;\nf(&x);"));
}
