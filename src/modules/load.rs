//! Recursive module graph loading and cycle detection.

use std::path::{Path, PathBuf};

use crate::ast::Program;

pub(super) struct ModuleLoader {
    pub(super) stack: Vec<PathBuf>,
}

impl ModuleLoader {
    pub(super) fn new() -> Self {
        Self { stack: Vec::new() }
    }

    pub(super) fn load_entry(mut self, path: &Path) -> Result<Program, String> {
        let entry = super::path::entry(path)?;
        let root = super::path::package_root(&entry)?;
        let module = self.load(&entry, &root)?;
        Ok(Program {
            imports: Vec::new(),
            exports: Vec::new(),
            stmts: module.stmts,
        })
    }

    pub(super) fn load(
        &mut self,
        path: &Path,
        root: &Path,
    ) -> Result<super::lower::LoadedModule, String> {
        if let Some(start) = self.stack.iter().position(|item| item == path) {
            let mut cycle = self.stack[start..].to_vec();
            cycle.push(path.to_owned());
            return Err(format_cycle(&cycle));
        }
        self.stack.push(path.to_owned());
        let result = super::load_body::load(self, path, root);
        self.stack.pop();
        result
    }
}

fn format_cycle(paths: &[PathBuf]) -> String {
    let chain = paths
        .iter()
        .map(|path| path.display().to_string())
        .collect::<Vec<_>>();
    format!("module import cycle: {}", chain.join(" -> "))
}
