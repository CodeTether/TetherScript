use crate::ir::{self, Block, Function, Instruction, Module, Operation, Terminator, ValueId};

#[test]
fn rejects_use_before_definition() {
    let function = Function {
        name: "broken".into(),
        params: vec![],
        blocks: vec![Block {
            label: "entry".into(),
            instructions: vec![Instruction {
                result: ValueId(0),
                operation: Operation::Move(ValueId(7)),
            }],
            terminator: Terminator::Return(ValueId(0)),
        }],
    };
    let error = ir::verify(&Module {
        functions: vec![function],
    })
    .unwrap_err();
    assert_eq!(
        error.to_string(),
        "invalid function `broken`: value %7 is used before definition"
    );
}
