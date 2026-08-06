//! Tests for infix-position operator mapping.
//!
//! The load-bearing case is `&`: it is the borrow sigil in prefix position and
//! bitwise AND in infix position, and only position distinguishes them.

use crate::ast::BinOp;

use super::infix_test_support::{let_operator, parses};

#[test]
fn infix_amp_is_bitwise_and_not_a_borrow() {
    assert_eq!(let_operator("let a = 12 & 10;"), BinOp::BitAnd);
}

#[test]
fn the_other_bitwise_tokens_map_to_their_operators() {
    assert_eq!(let_operator("let a = 12 | 10;"), BinOp::BitOr);
    assert_eq!(let_operator("let a = 12 ^ 10;"), BinOp::BitXor);
    assert_eq!(let_operator("let a = 1 << 4;"), BinOp::Shl);
    assert_eq!(let_operator("let a = 1 >> 4;"), BinOp::Shr);
}

#[test]
fn double_amp_is_still_logical_and() {
    assert_eq!(let_operator("let a = true && false;"), BinOp::And);
    assert_eq!(let_operator("let a = true || false;"), BinOp::Or);
}

#[test]
fn borrow_mut_and_borrow_arguments_still_parse() {
    assert!(parses("let mut n = 1;\nlet r = &mut n;"));
    assert!(parses("fn f(v) { v }\nlet x = 1;\nf(&x);"));
}
