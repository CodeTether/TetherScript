use std::fmt::Write;

use crate::ir::{Function, Terminator};

use super::operation;

pub(super) fn render(function: &Function) -> String {
    let mut output = String::new();
    let params = function
        .params
        .iter()
        .map(|param| format!("%{} {}", param.value.0, param.name))
        .collect::<Vec<_>>()
        .join(", ");
    writeln!(output, "fn @{}({params}) {{", function.name).unwrap();
    for block in &function.blocks {
        writeln!(output, "{}:", block.label).unwrap();
        for instruction in &block.instructions {
            writeln!(
                output,
                "  %{} = {}",
                instruction.result.0,
                operation::render(&instruction.operation)
            )
            .unwrap();
        }
        match block.terminator {
            Terminator::Return(value) => writeln!(output, "  return %{}", value.0).unwrap(),
        }
    }
    output.push('}');
    output
}
