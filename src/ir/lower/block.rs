use crate::ast::{Block, Stmt};
use crate::ir::{Constant, Operation, ValueId};

use super::{builder::Builder, expression, LowerError};

pub(super) fn lower(builder: &mut Builder, block: &Block) -> Result<ValueId, LowerError> {
    let mut result = None;
    for (index, statement) in block.stmts.iter().enumerate() {
        let last = index + 1 == block.stmts.len();
        match statement {
            Stmt::Let { name, value, .. } => {
                let value = expression::lower(builder, value)?;
                builder.bind(name, value);
            }
            Stmt::Expr { expr, terminated } => {
                let value = expression::lower(builder, expr)?;
                if last && !terminated {
                    result = Some(value);
                }
            }
            Stmt::FnDecl { name, .. } => {
                return Err(LowerError::new(
                    &builder.function,
                    format!("nested function `{name}` is not supported"),
                ))
            }
        }
    }
    Ok(result.unwrap_or_else(|| builder.emit(Operation::Constant(Constant::Nil))))
}
