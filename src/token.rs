//! Tokens — the atomic units the parser consumes.

use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Literals
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    Ident(String),

    // Keywords
    Fn,
    Let,
    Mut,
    Move,
    If,
    Else,
    While,
    For,
    In,
    Return,
    True,
    False,
    Nil,
    Panic,

    // Punctuation
    LParen,   // (
    RParen,   // )
    LBrace,   // {
    RBrace,   // }
    LBracket, // [
    RBracket, // ]
    Comma,    // ,
    Semi,     // ;
    Colon,    // :
    Dot,      // .
    Arrow,    // ->
    FatArrow, // =>
    Question, // ?

    // Operators
    Plus,    // +
    Minus,   // -
    Star,    // *
    Slash,   // /
    Percent, // %
    Assign,  // =
    Eq,      // ==
    NotEq,   // !=
    Lt,      // <
    Gt,      // >
    LtEq,    // <=
    GtEq,    // >=
    And,     // &&
    Or,      // ||
    Not,     // !
    Amp,     // & (borrow)
    AmpMut,  // &mut (lexed as two tokens; we fuse in parser)

    // Meta
    Newline,
    Eof,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// A token with source position, so error messages can point at something useful.
#[derive(Debug, Clone)]
pub struct Spanned {
    pub token: Token,
    pub line: usize,
    pub col: usize,
}
