//! Top-level synchronous and asynchronous function hoisting.

use std::cell::RefCell;
use std::rc::Rc;

use crate::ast::{Expr, Stmt};
use crate::value::{Env, FnObj, Value};

pub(super) fn statement(statement: &Stmt, environment: &Rc<RefCell<Env>>) {
    let (name, params, body, is_async) = match statement {
        Stmt::FnDecl { name, params, body } => (name, params, body, false),
        Stmt::Let {
            name,
            value: Expr::AsyncFn { params, body },
            ..
        } => (name, params, body, true),
        _ => return,
    };
    let function = Value::Fn(Rc::new(FnObj {
        params: params.clone(),
        body: body.clone(),
        is_async,
        closure: environment.clone(),
        name: Some(name.clone()),
    }));
    environment.borrow_mut().define(name, function, false);
}

pub(super) fn is_declaration(statement: &Stmt) -> bool {
    matches!(statement, Stmt::FnDecl { .. })
        || matches!(
            statement,
            Stmt::Let {
                value: Expr::AsyncFn { .. },
                ..
            }
        )
}
