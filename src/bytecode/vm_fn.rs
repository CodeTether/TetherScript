//! Captured bytecode function values.

use std::cell::RefCell;
use std::rc::Rc;

use crate::bytecode::FnProto;
use crate::value::Env;

/// A bytecode function prototype paired with its lexical environment.
pub struct VmFnObj {
    pub proto: Rc<FnProto>,
    pub closure: Rc<RefCell<Env>>,
    pub name: Option<String>,
}
