//! Abstract syntax tree.
//!
//! TetherScript is expression-oriented: a block is an expression whose value is its
//! trailing expression (if any). So `if`, `while`, `{ ... }` are all exprs.

use std::rc::Rc;

/// A part of a string interpolation literal.
#[derive(Debug, Clone)]
pub enum InterpPart {
    /// Literal text segment.
    Lit(String),
    /// Expression to evaluate and interpolate.
    Expr(Box<Expr>),
}

#[derive(Debug, Clone)]
pub enum Expr {
    // Literals
    Int(i64),
    Float(f64),
    Str(String),
    Bytes(Vec<u8>),
    Bool(bool),
    Nil,

    // Variable reference
    Ident(String),

    // Binary / unary
    Binary {
        op: BinOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    Unary {
        op: UnOp,
        rhs: Box<Expr>,
    },

    // Compound
    List(Vec<Expr>),
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
    },
    Index {
        target: Box<Expr>,
        index: Box<Expr>,
    },
    Field {
        target: Box<Expr>,
        name: String,
    },
    Method {
        target: Box<Expr>,
        name: String,
        args: Vec<Expr>,
    },

    // Ownership operators
    Move(Box<Expr>),      // `move x`
    Borrow(Box<Expr>),    // `&x`
    BorrowMut(Box<Expr>), // `&mut x`

    // Control flow (all expressions)
    If {
        cond: Box<Expr>,
        then_branch: Box<Block>,
        else_branch: Option<Box<Block>>,
    },
    While {
        cond: Box<Expr>,
        body: Box<Block>,
    },
    For {
        name: String,
        iter: Box<Expr>,
        body: Box<Block>,
    },
    Block(Box<Block>),

    // Function literal (anonymous)
    Fn {
        params: Vec<String>,
        body: Rc<Block>,
    },

    // `return expr` — unwinds to enclosing fn
    Return(Option<Box<Expr>>),

    // `panic "msg"` — unconditional halt
    Panic(Box<Expr>),

    // Async/concurrency expressions.
    AsyncFn {
        params: Vec<String>,
        body: Rc<Block>,
    },
    Await(Box<Expr>),
    Spawn(Box<Expr>),
    Join(Vec<Expr>),

    // `expr?` — if expr is Err(e), short-circuit out of the enclosing fn
    // by returning Err(e); if Ok(v), evaluate to v. See interp for semantics.
    Try(Box<Expr>),

    // String interpolation: `"hello, {name}"`
    StringInterp(Vec<InterpPart>),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    NotEq,
    Lt,
    Gt,
    LtEq,
    GtEq,
    And,
    Or,
    Assign,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnOp {
    Neg,
    Not,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    /// `let [mut] name = expr`
    Let {
        name: String,
        mutable: bool,
        value: Expr,
    },
    /// Bare expression statement (with or without trailing `;`).
    /// `terminated` = true means there was a semicolon, so the block should
    /// NOT use this expr as its value.
    Expr { expr: Expr, terminated: bool },
    /// `fn name(params) { body }` — sugar for `let name = fn(...) { ... }`,
    /// but we keep it as a distinct form so top-level fns can be hoisted.
    FnDecl {
        name: String,
        params: Vec<String>,
        body: Rc<Block>,
    },
}

#[derive(Debug, Clone)]
pub struct Block {
    pub stmts: Vec<Stmt>,
}

/// A file-relative module import with a local namespace alias.
///
/// # Examples
///
/// ```
/// use tetherscript::ast::ImportDecl;
/// let import = ImportDecl {
///     path: "./math.tether".into(),
///     alias: "math".into(),
/// };
/// assert_eq!(import.alias, "math");
/// ```
#[derive(Debug, Clone)]
pub struct ImportDecl {
    /// File path as written in the importing source file.
    pub path: String,
    /// Local namespace binding used to access exported values.
    pub alias: String,
}

/// Parsed source file with compile-time module metadata and executable statements.
///
/// # Examples
///
/// ```
/// use tetherscript::ast::Program;
/// let program = Program { imports: vec![], exports: vec![], stmts: vec![] };
/// assert!(program.stmts.is_empty());
/// ```
#[derive(Debug, Clone)]
pub struct Program {
    /// File-relative imports declared by this source file.
    pub imports: Vec<ImportDecl>,
    /// Top-level binding names exposed to importers.
    pub exports: Vec<String>,
    /// Executable declarations and statements.
    pub stmts: Vec<Stmt>,
}
