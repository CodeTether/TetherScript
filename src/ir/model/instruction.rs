use super::{Operation, ValueId};

/// Binds an operation result to an SSA identifier.
///
/// # Examples
///
/// ```
/// use tetherscript::ir::{Constant, Instruction, Operation, ValueId};
/// let instruction = Instruction {
///     result: ValueId(0),
///     operation: Operation::Constant(Constant::Int(1)),
/// };
/// assert_eq!(instruction.result, ValueId(0));
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct Instruction {
    pub result: ValueId,
    pub operation: Operation,
}
