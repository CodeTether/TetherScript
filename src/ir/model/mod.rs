mod block;
mod function;
mod instruction;
mod module;
mod operation;
mod value;

pub use block::{Block, Terminator};
pub use function::{Function, Parameter};
pub use instruction::Instruction;
pub use module::Module;
pub use operation::{BinaryOp, Operation};
pub use value::{Constant, ValueId};
