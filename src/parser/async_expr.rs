//! Async declarations and concurrency expressions.

use std::rc::Rc;

use crate::ast::{Expr, Stmt};
use crate::token::Token;

use super::{ParseError, Parser};

impl Parser {
    pub(super) fn parse_async_fn_decl(&mut self) -> Result<Stmt, ParseError> {
        self.bump();
        self.expect(&Token::Fn, "`fn` after `async`")?;
        let name = match self.bump().token {
            Token::Ident(name) => name,
            other => return Err(self.error(format!("expected fn name, got {other:?}"))),
        };
        let params = self.parse_params()?;
        let body = Rc::new(self.parse_block()?);
        Ok(Stmt::Let {
            name,
            mutable: false,
            value: Expr::AsyncFn { params, body },
        })
    }

    pub(super) fn parse_async_fn(&mut self) -> Result<Expr, ParseError> {
        self.bump();
        self.expect(&Token::Fn, "`fn` after `async`")?;
        let params = self.parse_params()?;
        let body = Rc::new(self.parse_block()?);
        Ok(Expr::AsyncFn { params, body })
    }
}
