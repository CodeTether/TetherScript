//! Runtime callbacks and scheduled bytecode invocation.

mod invoke;

use crate::value::{Runtime, Value};

use super::VM;

impl Runtime for VM {
    fn invoke(&mut self, callee: &Value, args: &[Value]) -> Result<Value, String> {
        self.invoke_mode(callee, args, false)
    }

    fn invoke_scheduled(&mut self, callee: &Value, args: &[Value]) -> Result<Value, String> {
        self.invoke_mode(callee, args, true)
    }

    fn global_defined(&self, name: &str) -> bool {
        self.globals.borrow().contains(name)
    }
}
