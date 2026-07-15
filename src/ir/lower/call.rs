use crate::ast::Expr;
use crate::ir::{Operation, ValueId};

use super::{builder::Builder, expression, LowerError};

pub(super) fn lower(
    builder: &mut Builder,
    callee: &Expr,
    args: &[Expr],
) -> Result<ValueId, LowerError> {
    let Expr::Ident(name) = callee else {
        return Err(LowerError::new(
            &builder.function,
            "only named calls are supported",
        ));
    };
    let values = args
        .iter()
        .map(|arg| expression::lower(builder, arg))
        .collect::<Result<_, _>>()?;
    Ok(builder.emit(Operation::Call {
        callee: name.clone(),
        args: values,
    }))
}
