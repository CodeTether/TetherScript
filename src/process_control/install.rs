use std::rc::Rc;

use crate::system;
use crate::value::{Env, NativeFn, NativeFunc, Value};

use super::{kill, list};

pub(crate) fn install(env: &mut Env) {
    define(env, "process_run", None, system::process_run);
    define(env, "process_args", Some(0), |_args| system::process_args());
    define(env, "process_pid", Some(0), |_args| system::process_pid());
    define(env, "process_platform", Some(0), |_args| {
        system::process_platform()
    });
    define(env, "process_arch", Some(0), |_args| system::process_arch());
    define(env, "process_list", Some(0), |_args| list());
    define(env, "process_kill", None, kill);
}

fn define(env: &mut Env, name: &str, arity: Option<usize>, call: fn(&[Value]) -> Value) {
    let func = NativeFunc::Pure(Box::new(move |args| Ok(call(args))));
    let native = NativeFn {
        name: name.into(),
        arity,
        func,
    };
    env.define(name, Value::Native(Rc::new(native)), false);
}
