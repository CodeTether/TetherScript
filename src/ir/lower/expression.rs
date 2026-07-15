use crate::ast::Expr;
use crate::ir::{Constant, Operation, ValueId};

use super::{builder::Builder, call, expression_kind, operator, LowerError};

pub(super) fn lower(builder: &mut Builder, expression: &Expr) -> Result<ValueId, LowerError> {
    let operation = match expression {
        Expr::Int(value) => Operation::Constant(Constant::Int(*value)),
        Expr::Float(value) => Operation::Constant(Constant::Float(*value)),
        Expr::Str(value) => Operation::Constant(Constant::Str(value.clone())),
        Expr::Bool(value) => Operation::Constant(Constant::Bool(*value)),
        Expr::Nil => Operation::Constant(Constant::Nil),
        Expr::Ident(name) => return builder.resolve(name),
        Expr::Move(inner) => Operation::Move(lower(builder, inner)?),
        Expr::Binary { op, lhs, rhs } => Operation::Binary {
            op: operator::lower(&builder.function, *op)?,
            lhs: lower(builder, lhs)?,
            rhs: lower(builder, rhs)?,
        },
        Expr::Call { callee, args } => return call::lower(builder, callee, args),
        other => {
            return Err(LowerError::new(
                &builder.function,
                format!("unsupported expression `{}`", expression_kind::of(other)),
            ))
        }
    };
    Ok(builder.emit(operation))
}
