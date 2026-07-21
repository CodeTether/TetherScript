//! Top-level program parsing, including module declarations.

use super::*;

impl Parser {
    pub fn parse_program(&mut self) -> Result<Program, ParseError> {
        let mut stmts = Vec::new();
        let mut imports = Vec::new();
        let mut exports = Vec::new();
        self.skip_newlines();
        while !self.at_end() {
            match self.peek() {
                Token::Import => imports.push(self.parse_import()?),
                Token::Export => exports.push(self.parse_export()?),
                _ => stmts.push(self.parse_stmt()?),
            }
            self.skip_newlines();
        }
        Ok(Program {
            imports,
            exports,
            stmts,
        })
    }

    fn parse_import(&mut self) -> Result<ImportDecl, ParseError> {
        self.bump();
        let path = match self.bump().token {
            Token::Str(value) => value,
            other => return Err(self.error(format!("expected import path, got {other:?}"))),
        };
        self.expect(&Token::As, "`as`")?;
        let alias = self.expect_ident("module alias")?;
        self.consume_stmt_end();
        Ok(ImportDecl { path, alias })
    }

    fn parse_export(&mut self) -> Result<String, ParseError> {
        self.bump();
        let name = self.expect_ident("exported binding")?;
        self.consume_stmt_end();
        Ok(name)
    }

    fn expect_ident(&mut self, what: &str) -> Result<String, ParseError> {
        match self.bump().token {
            Token::Ident(value) => Ok(value),
            other => Err(self.error(format!("expected {what}, got {other:?}"))),
        }
    }
}
