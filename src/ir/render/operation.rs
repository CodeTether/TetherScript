use crate::ir::{BinaryOp, Operation};

use super::constant;

pub(super) fn render(operation: &Operation) -> String {
    match operation {
        Operation::Constant(value) => constant::render(value),
        Operation::Binary { op, lhs, rhs } => {
            format!("value.{} %{}, %{}", binary(*op), lhs.0, rhs.0)
        }
        Operation::Move(value) => format!("ownership.move %{}", value.0),
        Operation::Call { callee, args } => {
            let args = args
                .iter()
                .map(|value| format!("%{}", value.0))
                .collect::<Vec<_>>()
                .join(", ");
            format!("call @{callee}({args})")
        }
    }
}

fn binary(operator: BinaryOp) -> &'static str {
    match operator {
        BinaryOp::Add => "add",
        BinaryOp::Sub => "sub",
        BinaryOp::Mul => "mul",
        BinaryOp::Div => "div",
        BinaryOp::Mod => "mod",
        BinaryOp::Eq => "eq",
        BinaryOp::NotEq => "ne",
        BinaryOp::Lt => "lt",
        BinaryOp::Gt => "gt",
        BinaryOp::LtEq => "le",
        BinaryOp::GtEq => "ge",
    }
}
