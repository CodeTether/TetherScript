//! Installation of the language `resource` namespace.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::value::{Env, Value};

use super::{factory_memory as memory, factory_os as os, native};

pub(crate) fn install(env: &Rc<RefCell<Env>>) {
    let mut module = HashMap::new();
    native::insert(&mut module, "file", 2, os::file);
    native::insert(&mut module, "child_process", 2, os::child);
    native::insert(&mut module, "tcp_connect", 3, os::tcp_connect);
    native::insert(&mut module, "tcp_listen", 2, os::tcp_listen);
    native::insert(&mut module, "request_body", 2, memory::request_body);
    native::insert(&mut module, "response_writer", 1, memory::response_writer);
    native::insert(&mut module, "task", 0, memory::task);
    native::insert(&mut module, "timer", 1, memory::timer);
    native::insert(&mut module, "channel", 1, memory::channel);
    env.borrow_mut()
        .define("resource", Value::Map(Rc::new(RefCell::new(module))), false);
}
