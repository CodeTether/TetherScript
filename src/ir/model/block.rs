use super::{Instruction, ValueId};

/// Ends a basic block and transfers control.
///
/// # Examples
///
/// ```
/// use tetherscript::ir::{Terminator, ValueId};
/// assert_eq!(Terminator::Return(ValueId(0)), Terminator::Return(ValueId(0)));
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Terminator {
    Return(ValueId),
}

/// Contains straight-line SSA instructions followed by one terminator.
///
/// # Examples
///
/// ```
/// use tetherscript::ir::{Block, Terminator, ValueId};
/// let block = Block { label: "entry".into(), instructions: vec![],
///     terminator: Terminator::Return(ValueId(0)) };
/// assert_eq!(block.label, "entry");
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct Block {
    pub label: String,
    pub instructions: Vec<Instruction>,
    pub terminator: Terminator,
}
