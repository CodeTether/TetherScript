mod block;
mod builder;
mod call;
mod error;
mod expression;
mod expression_kind;
mod function;
mod operator;
mod scope;

use crate::ast::{Program, Stmt};
use crate::ir::Module;

pub use error::LowerError;

/// Lowers a parsed program into backend-independent Tether IR.
///
/// # Arguments
///
/// * `program` — Parsed tetherscript program containing function declarations.
///
/// # Returns
///
/// A module whose functions use SSA value identifiers and explicit returns.
///
/// # Errors
///
/// Returns [`LowerError`] when the program is outside the initial straight-line subset.
pub fn lower_program(program: &Program) -> Result<Module, LowerError> {
    let mut functions = Vec::new();
    for statement in &program.stmts {
        match statement {
            Stmt::FnDecl { name, params, body } => {
                functions.push(function::lower(name, params, body)?);
            }
            _ => {
                return Err(LowerError::new(
                    "<module>",
                    "top-level executable statements are outside the initial IR subset",
                ))
            }
        }
    }
    Ok(Module { functions })
}
