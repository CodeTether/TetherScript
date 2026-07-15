//! # Tether Intermediate Representation
//!
//! This module defines the backend-independent compiler layer between the
//! tetherscript AST and future bytecode or native machine backends.
//!
//! [`lower_program`] converts the supported straight-line AST subset to SSA-like
//! values, [`verify`] enforces structural invariants, and [`render`] produces a
//! stable textual representation for inspection and tests.
//!
//! # Usage
//!
//! ```
//! use tetherscript::{ast::{Block, Expr, Program, Stmt}, ir};
//! use std::rc::Rc;
//!
//! let program = Program { stmts: vec![Stmt::FnDecl {
//!     name: "answer".into(), params: vec![],
//!     body: Rc::new(Block { stmts: vec![Stmt::Expr {
//!         expr: Expr::Int(42), terminated: false,
//!     }] }),
//! }] };
//! let module = ir::lower_program(&program).unwrap();
//! ir::verify(&module).unwrap();
//! assert!(ir::render(&module).contains("const.int 42"));
//! ```

mod lower;
mod model;
mod render;
mod verify;

pub use lower::{lower_program, LowerError};
pub use model::*;
pub use render::render;
pub use verify::{verify, VerifyError};

#[cfg(test)]
mod tests;
