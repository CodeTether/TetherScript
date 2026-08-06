//! Integration coverage for the `&`-is-not-bitwise-AND diagnostic.
//!
//! Locks in the fix for a silent-wrong-answer bug: `let a = 12 & 10;` used to
//! parse as `let a = 12` followed by a discarded `&10` borrow, so the program
//! printed `12` and exited 0. There is no bitwise AND in tetherscript, so `&`
//! between two expressions is now a named parse error.

use tetherscript::lexer::Lexer;
use tetherscript::parser::Parser;

/// Parse `source`, returning the parse error message.
fn parse_error(source: &str) -> String {
    let tokens = Lexer::new(source).tokenize().expect("source should lex");
    Parser::new(tokens)
        .parse_program()
        .expect_err("source should be rejected")
        .msg
}

/// Report whether `source` lexes and parses cleanly.
fn parses(source: &str) -> bool {
    match Lexer::new(source).tokenize() {
        Ok(tokens) => Parser::new(tokens).parse_program().is_ok(),
        Err(_) => false,
    }
}

#[test]
fn amp_between_integers_is_a_named_error() {
    let msg = parse_error("fn main() { let a = 12 & 10; a }");

    assert!(msg.contains("`&` is not a binary operator"), "got: {msg}");
    assert!(msg.contains("bitwise AND"), "got: {msg}");
}

#[test]
fn the_error_names_both_alternatives() {
    let msg = parse_error("fn main() { let a = 12 & 10; a }");

    assert!(msg.contains("`&&`"), "got: {msg}");
    assert!(msg.contains("borrow"), "got: {msg}");
}

#[test]
fn amp_between_identifiers_is_rejected() {
    let msg = parse_error("fn main() { let x = 1 let y = x & x y }");

    assert!(msg.contains("`&` is not a binary operator"), "got: {msg}");
}

#[test]
fn amp_after_a_call_is_rejected() {
    let msg = parse_error("fn f() { 1 } fn main() { let y = f() & 2 y }");

    assert!(msg.contains("`&` is not a binary operator"), "got: {msg}");
}

#[test]
fn prefix_borrow_and_borrow_mut_still_parse() {
    assert!(parses("fn main() { let xs = [1] let r = &xs r.len() }"));
    assert!(parses("fn main() { let mut n = 1 let r = &mut n r }"));
}

#[test]
fn logical_and_and_borrow_arguments_still_parse() {
    assert!(parses("fn main() { true && false }"));
    assert!(parses("fn f(v) { v } fn main() { let x = 1 f(&x) }"));
}
