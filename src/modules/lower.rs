//! Lower a loaded module into an isolated namespace expression.

use crate::ast::{BinOp, Block, Expr, Stmt};

pub(super) struct LoadedModule {
    pub(super) stmts: Vec<Stmt>,
    pub(super) exports: Vec<String>,
}

impl LoadedModule {
    pub(super) fn into_namespace(self) -> Expr {
        let internal = "\0module_exports".to_string();
        let mut stmts = self.stmts;
        stmts.push(Stmt::Let {
            name: internal.clone(),
            mutable: false,
            value: Expr::Call {
                callee: Box::new(Expr::Ident("map".into())),
                args: Vec::new(),
            },
        });
        for name in self.exports {
            stmts.push(export_assignment(&internal, &name));
        }
        stmts.push(Stmt::Expr {
            expr: Expr::Ident(internal),
            terminated: false,
        });
        Expr::Block(Box::new(Block { stmts }))
    }
}

fn export_assignment(module: &str, name: &str) -> Stmt {
    Stmt::Expr {
        expr: Expr::Binary {
            op: BinOp::Assign,
            lhs: Box::new(Expr::Field {
                target: Box::new(Expr::Ident(module.into())),
                name: name.into(),
            }),
            rhs: Box::new(Expr::Ident(name.into())),
        },
        terminated: true,
    }
}
