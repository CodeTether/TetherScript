//! Load and validate the declarations in one module.

use std::path::Path;

pub(super) fn load(
    loader: &mut super::load::ModuleLoader,
    path: &Path,
    root: &Path,
) -> Result<super::lower::LoadedModule, String> {
    let program = super::parse::file(path)?;
    let mut names = super::names::Names::new(path);
    let mut stmts = Vec::new();
    for stmt in &program.stmts {
        if let Some(name) = super::names::declared(stmt) {
            names.declare(name)?;
        }
    }
    for import in program.imports {
        names.declare(&import.alias)?;
        let target = super::path::import(path, root, &import.path)?;
        let value = loader.load(&target, root)?.into_namespace();
        stmts.push(crate::ast::Stmt::Let {
            name: import.alias,
            mutable: false,
            value,
        });
    }
    stmts.extend(program.stmts);
    let mut unique = std::collections::HashSet::new();
    for name in &program.exports {
        if !unique.insert(name) {
            return Err(format!("{}: duplicate export `{name}`", path.display()));
        }
        if !names.contains(name) {
            return Err(format!(
                "{}: export `{name}` is not declared",
                path.display()
            ));
        }
    }
    Ok(super::lower::LoadedModule {
        stmts,
        exports: program.exports,
    })
}
