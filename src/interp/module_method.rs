//! Method dispatch for values and imported module namespaces.

use std::cell::RefCell;
use std::rc::Rc;

use crate::ast::Expr;
use crate::value::{Env, Value};

use super::{call_capability_method, call_method, EvalResult, InterpRuntime, Interpreter, Unwind};

impl Interpreter {
    pub(super) fn eval_method(
        &self,
        target: &Expr,
        name: &str,
        args: &[Expr],
        env: &Rc<RefCell<Env>>,
    ) -> EvalResult {
        let target = self.eval(target, env)?;
        let args = args
            .iter()
            .map(|arg| self.eval(arg, env))
            .collect::<Result<Vec<_>, _>>()?;
        if let Value::Map(map) = &target {
            if let Some(callee) = map.borrow().get(name).cloned() {
                return self.call_owned(&callee, args);
            }
        }
        if let Value::Capability(capability) = &target {
            let mut runtime = InterpRuntime { interp: self };
            return call_capability_method(capability, name, &args, &mut runtime)
                .map_err(Unwind::Error);
        }
        call_method(&target, name, &args).map_err(Unwind::Error)
    }
}
