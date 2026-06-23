//! Instruction display helpers.

use crate::bytecode::Instr;

pub(super) fn render(pc: usize, instr: &Instr) -> String {
    match target(pc, instr) {
        Some(target) => format!("{pc:04}  {instr:?} -> {target:04}"),
        None => format!("{pc:04}  {instr:?}"),
    }
}

fn target(pc: usize, instr: &Instr) -> Option<i32> {
    let next = pc as i32 + 1;
    match instr {
        Instr::Jump(offset)
        | Instr::JumpIfFalse(offset)
        | Instr::JumpIfFalseKeep(offset)
        | Instr::JumpIfTrueKeep(offset)
        | Instr::ForNext(_, offset) => Some(next + *offset),
        _ => None,
    }
}
