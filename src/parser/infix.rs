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
/// `&` is bitwise AND here. Position is what disambiguates it from the borrow
/// sigil: `parse_prefix` claims a leading `&`, so by the time this table is
/// consulted there is a left operand and the only sensible reading is the
/// binary operator. The ladder follows Rust's, so `|` binds looser than `^`,
/// which binds looser than `&`, which binds looser than the shifts.
pub(super) fn infix_prec(t: &Token) -> Prec {
    match t {
        Token::Assign => Prec::Assign,
        Token::Or => Prec::Or,
        Token::And => Prec::And,
        Token::Eq | Token::NotEq => Prec::Equality,
        Token::Lt | Token::Gt | Token::LtEq | Token::GtEq => Prec::Compare,
        Token::Pipe => Prec::BitOr,
        Token::Caret => Prec::BitXor,
        Token::Amp => Prec::BitAnd,
        Token::Shl | Token::Shr => Prec::Shift,
        Token::Plus | Token::Minus => Prec::Term,
        Token::Star | Token::Slash | Token::Percent => Prec::Factor,
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
        Token::Amp => BinOp::BitAnd,
        Token::Pipe => BinOp::BitOr,
        Token::Caret => BinOp::BitXor,
        Token::Shl => BinOp::Shl,
        Token::Shr => BinOp::Shr,
        Token::Assign => BinOp::Assign,
        _ => return None,
    })
}

/// Explain why `tok` cannot join two expressions.
pub(super) fn infix_error_message(tok: &Token) -> String {
    format!("unexpected infix token: {tok:?}")
}
