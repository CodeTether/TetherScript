use std::collections::HashSet;

use crate::ir::{Function, Operation, Terminator};

use super::{definitions::Definitions, operands, VerifyError};

pub(super) fn verify(function: &Function) -> Result<(), VerifyError> {
    if function.name.is_empty() {
        return fail(function, "function name is empty");
    }
    if function.blocks.len() != 1 || function.blocks[0].label != "entry" {
        return fail(function, "initial IR requires exactly one `entry` block");
    }
    let mut names = HashSet::new();
    let mut definitions = Definitions::new(&function.name);
    for parameter in &function.params {
        if !names.insert(&parameter.name) {
            return fail(
                function,
                format!("duplicate parameter `{}`", parameter.name),
            );
        }
        definitions.define(parameter.value)?;
    }
    let block = &function.blocks[0];
    for instruction in &block.instructions {
        for operand in operands::of(&instruction.operation) {
            definitions.require(operand)?;
        }
        if let Operation::Call { callee, .. } = &instruction.operation {
            if callee.is_empty() {
                return fail(function, "call target is empty");
            }
        }
        definitions.define(instruction.result)?;
    }
    match block.terminator {
        Terminator::Return(value) => definitions.require(value),
    }
}

fn fail<T>(function: &Function, message: impl Into<String>) -> Result<T, VerifyError> {
    Err(VerifyError::function(&function.name, message))
}
