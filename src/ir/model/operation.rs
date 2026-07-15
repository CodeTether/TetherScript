use super::{Constant, ValueId};

/// Binary operation encoded by Tether IR.
///
/// # Examples
///
/// ```
/// use tetherscript::ir::BinaryOp;
/// assert_eq!(BinaryOp::Add, BinaryOp::Add);
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    NotEq,
    Lt,
    Gt,
    LtEq,
    GtEq,
}

/// Computes one SSA value.
///
/// Variants cover constants, dynamic binary operations, value movement, and
/// named calls. More specialized operations can be introduced by later passes.
#[derive(Clone, Debug, PartialEq)]
pub enum Operation {
    Constant(Constant),
    Binary {
        op: BinaryOp,
        lhs: ValueId,
        rhs: ValueId,
    },
    Move(ValueId),
    Call {
        callee: String,
        args: Vec<ValueId>,
    },
}
