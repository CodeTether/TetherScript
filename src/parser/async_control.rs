//! Await, spawn, and ordered join parsing.

use crate::ast::Expr;
use crate::token::Token;

use super::{ParseError, Parser};

impl Parser {
    pub(super) fn parse_await(&mut self) -> Result<Expr, ParseError> {
        self.bump();
        Ok(Expr::Await(Box::new(self.parse_expr()?)))
    }

    pub(super) fn parse_spawn(&mut self) -> Result<Expr, ParseError> {
        self.bump();
        Ok(Expr::Spawn(Box::new(self.parse_expr()?)))
    }

    pub(super) fn parse_join(&mut self) -> Result<Expr, ParseError> {
        self.bump();
        self.expect(&Token::LParen, "`(` after `join`")?;
        let mut handles = Vec::new();
        self.skip_newlines();
        while !self.check(&Token::RParen) {
            handles.push(self.parse_expr()?);
            self.skip_newlines();
            if !self.eat(&Token::Comma) {
                break;
            }
            self.skip_newlines();
        }
        self.expect(&Token::RParen, "`)`")?;
        Ok(Expr::Join(handles))
    }
}
