use super::Function;

/// Collection of functions passed between compiler stages.
///
/// # Examples
///
/// ```
/// use tetherscript::ir::Module;
/// let module = Module::default();
/// assert!(module.functions.is_empty());
/// ```
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Module {
    pub functions: Vec<Function>,
}
