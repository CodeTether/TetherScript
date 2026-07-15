use crate::ir::{Block, Constant, Function, Instruction, Operation, Terminator, ValueId};

pub(super) fn empty_function(name: &str) -> Function {
    Function {
        name: name.into(),
        params: vec![],
        blocks: vec![Block {
            label: "entry".into(),
            instructions: vec![Instruction {
                result: ValueId(0),
                operation: Operation::Constant(Constant::Nil),
            }],
            terminator: Terminator::Return(ValueId(0)),
        }],
    }
}
