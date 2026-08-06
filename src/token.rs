//! Tokens — the atomic units the parser consumes.
//!
//! This module defines the [`Token`] enum, which represents every possible
//! lexical unit in tetherscript, and the [`Spanned`] wrapper which attaches
//! source location information for error reporting.

/// A segment inside an interpolated string literal.
///
/// Interpolated strings (e.g., `"Hello, {name}"`) are lexed as a sequence
/// of raw text literals and expression holes.
#[derive(Debug, Clone, PartialEq)]
pub enum InterpSegment {
    /// Raw text between `{}` holes.
    Lit(String),
    /// Source text of an expression inside `{}`.
    Expr(String),
}

/// Every possible lexical unit in the tetherscript language.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Literals
    /// An integer literal (e.g., `42`, `-10`).
    Int(i64),
    /// A floating-point literal (e.g., `3.14`, `2.0`).
    Float(f64),
    /// A standard string literal (e.g., `"hello"`).
    Str(String),
    /// An interpolated string: `"hello, {name}"`.
    StrInterp(Vec<InterpSegment>),
    /// A byte array literal.
    Bytes(Vec<u8>),
    /// A boolean literal (`true` or `false`).
    Bool(bool),
    /// An identifier (variable or function name).
    Ident(String),

    // Keywords
    /// `fn` keyword for function definitions.
    Fn,
    /// `let` keyword for variable declarations.
    Let,
    /// `mut` keyword for mutable declarations.
    Mut,
    /// `move` keyword for ownership transfer.
    Move,
    /// `if` keyword for conditional branching.
    If,
    /// `else` keyword for conditional branching.
    Else,
    /// `while` keyword for loops.
    While,
    /// `for` keyword for iteration.
    For,
    /// `in` keyword for iteration.
    In,
    /// `return` keyword for function exits.
    Return,
    /// `nil` literal for empty/null values.
    Nil,
    /// `panic` keyword for runtime errors.
    Panic,
    /// `async` keyword for asynchronous functions.
    Async,
    /// `await` keyword for suspending execution.
    Await,
    /// `spawn` keyword for concurrent execution.
    Spawn,
    /// `join` keyword for synchronizing execution.
    Join,
    Import,
    Export,
    As,

    // Punctuation
    /// Left parenthesis `(`.
    LParen,
    /// Right parenthesis `)`.
    RParen,
    /// Left brace `{`.
    LBrace,
    /// Right brace `}`.
    RBrace,
    /// Left bracket `[`.
    LBracket,
    /// Right bracket `]`.
    RBracket,
    /// Comma `,`.
    Comma,
    /// Semicolon `;`.
    Semi,
    /// Colon `:`.
    Colon,
    /// Dot `.`.
    Dot,
    /// Arrow `->`.
    Arrow,
    /// Fat arrow `=>`.
    FatArrow,
    /// Question mark `?`.
    Question,

    // Operators
    /// Plus `+`.
    Plus,
    /// Minus `-`.
    Minus,
    /// Multiplication `*`.
    Star,
    /// Division `/`.
    Slash,
    /// Modulo `%`.
    Percent,
    /// Assignment `=`.
    Assign,
    /// Equality `==`.
    Eq,
    /// Inequality `!=`.
    NotEq,
    /// Less than `<`.
    Lt,
    /// Greater than `>`.
    Gt,
    /// Less than or equal to `<=`.
    LtEq,
    /// Greater than or equal to `>=`.
    GtEq,
    /// Logical AND `&&`.
    And,
    /// Logical OR `||`.
    Or,
    /// Logical NOT `!`.
    Not,
    /// Borrow `&` in prefix position, bitwise AND in infix position.
    Amp,
    /// Bitwise OR `|`.
    Pipe,
    /// Bitwise XOR `^`.
    Caret,
    /// Bitwise NOT `~`.
    Tilde,
    /// Left shift `<<`.
    Shl,
    /// Right shift `>>`.
    Shr,

    // Meta
    /// Represents a line break in the source.
    Newline,
    /// End of file marker.
    Eof,
}

mod display;

/// A token with source position, so error messages can point at something useful.
///
/// This is the primary unit consumed by the [`crate::parser::Parser`].
#[derive(Debug, Clone)]
pub struct Spanned {
    /// The actual lexical token.
    pub token: Token,
    /// The 1-indexed line number in the source file.
    pub line: usize,
    /// The 1-indexed column number in the source file.
    pub col: usize,
}
