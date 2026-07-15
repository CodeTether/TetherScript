mod definitions;
mod error;
mod function;
mod operands;

use std::collections::HashSet;

use crate::ir::Module;

pub use error::VerifyError;

/// Checks that a Tether IR module satisfies structural and SSA invariants.
///
/// # Arguments
///
/// * `module` — Module to check before optimization or code generation.
///
/// # Returns
///
/// `Ok(())` when every function and value reference is well formed.
///
/// # Errors
///
/// Returns [`VerifyError`] with the function and violated invariant.
///
/// # Examples
///
/// ```
/// use tetherscript::ir::{verify, Module};
/// assert!(verify(&Module::default()).is_ok());
/// ```
pub fn verify(module: &Module) -> Result<(), VerifyError> {
    let mut names = HashSet::new();
    for item in &module.functions {
        if !names.insert(&item.name) {
            return Err(VerifyError::module(format!(
                "duplicate function `{}`",
                item.name
            )));
        }
        function::verify(item)?;
    }
    Ok(())
}
