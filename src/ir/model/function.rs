use super::{Block, ValueId};

/// Names an incoming function value and its SSA identifier.
///
/// # Examples
///
/// ```
/// use tetherscript::ir::{Parameter, ValueId};
/// let parameter = Parameter { name: "input".into(), value: ValueId(0) };
/// assert_eq!(parameter.name, "input");
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub value: ValueId,
}

/// Defines one lowered tetherscript function.
///
/// # Examples
///
/// ```
/// use tetherscript::ir::Function;
/// let function = Function { name: "main".into(), params: vec![], blocks: vec![] };
/// assert_eq!(function.name, "main");
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct Function {
    pub name: String,
    pub params: Vec<Parameter>,
    pub blocks: Vec<Block>,
}
