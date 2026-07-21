//! Module binding-name validation.

use std::collections::HashSet;
use std::path::Path;

use crate::ast::Stmt;

pub(super) struct Names<'a> {
    values: HashSet<String>,
    path: &'a Path,
}

impl<'a> Names<'a> {
    pub(super) fn new(path: &'a Path) -> Self {
        Self {
            values: HashSet::new(),
            path,
        }
    }

    pub(super) fn declare(&mut self, name: &str) -> Result<(), String> {
        if self.values.insert(name.to_owned()) {
            Ok(())
        } else {
            Err(format!(
                "{}: duplicate module binding `{name}`",
                self.path.display()
            ))
        }
    }

    pub(super) fn contains(&self, name: &str) -> bool {
        self.values.contains(name)
    }
}

pub(super) fn declared(stmt: &Stmt) -> Option<&str> {
    match stmt {
        Stmt::Let { name, .. } | Stmt::FnDecl { name, .. } => Some(name),
        _ => None,
    }
}
