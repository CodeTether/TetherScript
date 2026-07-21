//! Bytecode lowering for cooperative async expressions.

use std::rc::Rc;

use crate::ast::{Block, Expr, Stmt};
use crate::bytecode::Instr;

use super::Compiler;

pub(super) fn is_declaration(statement: &Stmt) -> bool {
    matches!(
        statement,
        Stmt::Let {
            value: Expr::AsyncFn { .. },
            ..
        }
    )
}

impl Compiler {
    pub(super) fn compile_async_fn(&mut self, params: &[String], body: &Rc<Block>) {
        let proto = Self::compile_fn(None, params, body, true);
        let index = self.chunk.protos.len() as u16;
        self.chunk.protos.push(Rc::new(proto));
        self.emit(Instr::MakeFn(index));
    }

    pub(super) fn compile_await(&mut self, expression: &Expr) {
        self.compile_expr(expression);
        self.emit(Instr::Await);
    }

    pub(super) fn compile_spawn(&mut self, expression: &Expr) {
        self.compile_expr(expression);
        self.emit(Instr::Spawn);
    }

    pub(super) fn compile_join(&mut self, handles: &[Expr]) {
        for handle in handles {
            self.compile_expr(handle);
        }
        self.emit(Instr::Join(handles.len() as u16));
    }
}
