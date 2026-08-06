//! Infix operator precedence and infix-position diagnostics.
//!
//! Split out of [`crate::parser`] so the Pratt loop's precedence table and its
//! error wording live in one focused place. Two concerns only: map a token to
//! its binding power, and explain why a token cannot appear in infix position.

use crate::ast::BinOp;
use crate::token::Token;

use super::Prec;

/// Binding power of `t` when it appears *after* an expression.
///
/// [`Prec::None`] means "not an infix operator", which ends the Pratt loop and
/// therefore ends the statement.
///
/// `&` is deliberately given the maximum binding power even though it is not a
/// binary operator. tetherscript has no bitwise AND, so `a & b` is always a
/// mistake; ranking it highest guarantees [`infix_error_message`] reports it in
/// every infix position instead of letting the loop end and silently reinterpret
/// `& b` as a borrow statement.
pub(super) fn infix_prec(t: &Token) -> Prec {
    match t {
        Token::Assign => Prec::Assign,
        Token::Or => Prec::Or,
        Token::And => Prec::And,
        Token::Eq | Token::NotEq => Prec::Equality,
        Token::Lt | Token::Gt | Token::LtEq | Token::GtEq => Prec::Compare,
        Token::Plus | Token::Minus => Prec::Term,
        Token::Star | Token::Slash | Token::Percent => Prec::Factor,
        Token::Amp => Prec::Call,
        Token::LParen | Token::LBracket | Token::Dot | Token::Question => Prec::Call,
        _ => Prec::None,
    }
}

/// Map an infix token to its binary operator, or `None` if it is not one.
pub(super) fn infix_binop(t: &Token) -> Option<BinOp> {
    Some(match t {
        Token::Plus => BinOp::Add,
        Token::Minus => BinOp::Sub,
        Token::Star => BinOp::Mul,
        Token::Slash => BinOp::Div,
        Token::Percent => BinOp::Mod,
        Token::Eq => BinOp::Eq,
        Token::NotEq => BinOp::NotEq,
        Token::Lt => BinOp::Lt,
        Token::Gt => BinOp::Gt,
        Token::LtEq => BinOp::LtEq,
        Token::GtEq => BinOp::GtEq,
        Token::And => BinOp::And,
        Token::Or => BinOp::Or,
        Token::Assign => BinOp::Assign,
        _ => return None,
    })
}

/// Explain why `tok` cannot join two expressions.
///
/// `&` gets a dedicated message because it is the one token that reads like a
/// binary operator to anyone arriving from C, Python, or Rust's integer types.
pub(super) fn infix_error_message(tok: &Token) -> String {
    match tok {
        Token::Amp => "`&` is not a binary operator; tetherscript has no bitwise AND. \
             Use `&&` for logical and, or write `&value` as a prefix to borrow."
            .to_string(),
        other => format!("unexpected infix token: {:?}", other),
    }
}
