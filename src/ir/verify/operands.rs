use crate::ir::{Operation, ValueId};

pub(super) fn of(operation: &Operation) -> Vec<ValueId> {
    match operation {
        Operation::Constant(_) => vec![],
        Operation::Binary { lhs, rhs, .. } => vec![*lhs, *rhs],
        Operation::Move(value) => vec![*value],
        Operation::Call { args, .. } => args.clone(),
    }
}
