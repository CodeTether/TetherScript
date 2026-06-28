//! Command-line argument globals for script CLIs.

use std::cell::RefCell;
use std::rc::Rc;

use crate::value::{Env, NativeFn, NativeFunc, Value};

/// Install CLI argument helpers into a runtime global environment.
pub(crate) fn install(env: &Rc<RefCell<Env>>, args: &[String]) {
    let captured = args.to_vec();
    env.borrow_mut().define(
        "env_args",
        Value::Native(Rc::new(NativeFn {
            name: "env_args".into(),
            arity: Some(0),
            func: NativeFunc::Pure(Box::new(move |_| Ok(list(&captured)))),
        })),
        false,
    );
}

fn list(args: &[String]) -> Value {
    Value::List(Rc::new(RefCell::new(
        args.iter()
            .map(|arg| Value::Str(Rc::new(arg.clone())))
            .collect(),
    )))
}
