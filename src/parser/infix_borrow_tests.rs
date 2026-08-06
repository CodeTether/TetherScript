//! Tests that prefix `&` still parses as a borrow, not bitwise AND.

use crate::ast::Expr;

use super::infix_test_support::let_value;

#[test]
fn prefix_amp_is_still_a_borrow() {
    let value = let_value("let xs = [1];\nlet r = &xs;", 1);

    assert!(matches!(value, Expr::Borrow(_)), "got {value:?}");
}

#[test]
fn prefix_amp_mut_is_still_a_mutable_borrow() {
    let value = let_value("let mut n = 1;\nlet r = &mut n;", 1);

    assert!(matches!(value, Expr::BorrowMut(_)), "got {value:?}");
}
