use crate::ast::BinOp;
use crate::ir::BinaryOp;

use super::LowerError;

pub(super) fn lower(function: &str, operator: BinOp) -> Result<BinaryOp, LowerError> {
    let lowered = match operator {
        BinOp::Add => BinaryOp::Add,
        BinOp::Sub => BinaryOp::Sub,
        BinOp::Mul => BinaryOp::Mul,
        BinOp::Div => BinaryOp::Div,
        BinOp::Mod => BinaryOp::Mod,
        BinOp::Eq => BinaryOp::Eq,
        BinOp::NotEq => BinaryOp::NotEq,
        BinOp::Lt => BinaryOp::Lt,
        BinOp::Gt => BinaryOp::Gt,
        BinOp::LtEq => BinaryOp::LtEq,
        BinOp::GtEq => BinaryOp::GtEq,
        BinOp::And | BinOp::Or => {
            return Err(LowerError::new(
                function,
                "short-circuit operators require control-flow IR",
            ))
        }
        BinOp::Assign => {
            return Err(LowerError::new(
                function,
                "assignment requires mutable-slot IR",
            ))
        }
    };
    Ok(lowered)
}
