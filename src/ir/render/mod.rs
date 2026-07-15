mod constant;
mod function;
mod operation;

use crate::ir::Module;

/// Renders a module in the stable textual Tether IR format.
///
/// # Arguments
///
/// * `module` — Verified module to make human-readable.
///
/// # Returns
///
/// Text containing each function, basic block, SSA instruction, and terminator.
///
/// # Examples
///
/// ```
/// use tetherscript::ir::{render, Module};
/// assert_eq!(render(&Module::default()), "");
/// ```
pub fn render(module: &Module) -> String {
    module
        .functions
        .iter()
        .map(function::render)
        .collect::<Vec<_>>()
        .join("\n\n")
}
