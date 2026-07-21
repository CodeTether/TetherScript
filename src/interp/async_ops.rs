//! Tree-walker cooperative task expressions.

use std::cell::RefCell;
use std::rc::Rc;

use crate::ast::Expr;
use crate::scheduler::runtime;
use crate::value::{Env, Value};

use super::{resource_transfer, EvalResult, InterpRuntime, Interpreter, Unwind};

impl Interpreter {
    pub(super) fn eval_await(&self, expression: &Expr, env: &Rc<RefCell<Env>>) -> EvalResult {
        let task = self.eval(expression, env)?;
        let mut bridge = InterpRuntime { interp: self };
        runtime::await_value(&mut bridge, task).map_err(Unwind::Error)
    }

    pub(super) fn eval_spawn(&self, expression: &Expr, env: &Rc<RefCell<Env>>) -> EvalResult {
        runtime::spawn(self.eval(expression, env)?).map_err(Unwind::Error)
    }

    pub(super) fn eval_join(&self, expressions: &[Expr], env: &Rc<RefCell<Env>>) -> EvalResult {
        let tasks = expressions
            .iter()
            .map(|expression| self.eval(expression, env))
            .collect::<Result<Vec<Value>, Unwind>>()?;
        let mut bridge = InterpRuntime { interp: self };
        let values = runtime::join(&mut bridge, tasks).map_err(Unwind::Error)?;
        resource_transfer::list(values, "join result")
    }
}
