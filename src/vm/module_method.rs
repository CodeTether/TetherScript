//! Method dispatch for values and imported module namespaces.

use crate::interp::{call_capability_method, call_method};
use crate::value::{Runtime, Value};

use super::{Unwind, VM};

impl VM {
    pub(super) fn dispatch_method(
        &mut self,
        target: Value,
        name: &str,
        args: Vec<Value>,
    ) -> Result<(), Unwind> {
        let module_call = match &target {
            Value::Map(map) => map.borrow().get(name).cloned(),
            _ => None,
        };
        if let Some(callee) = module_call {
            return self.dispatch_call(callee, args);
        }
        let result = match &target {
            Value::Capability(capability) => {
                let capability = capability.clone();
                call_capability_method(&capability, name, &args, self as &mut dyn Runtime)?
            }
            _ => call_method(&target, name, &args)?,
        };
        self.stack.push(result);
        Ok(())
    }
}
