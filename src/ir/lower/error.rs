use std::fmt::{self, Display};

/// Reports why an AST construct cannot enter Tether IR yet.
///
/// # Examples
///
/// ```
/// use tetherscript::{ast::{Expr, Program, Stmt}, ir};
/// let program = Program { stmts: vec![Stmt::Expr {
///     expr: Expr::Int(1), terminated: false,
/// }] };
/// assert!(ir::lower_program(&program).is_err());
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LowerError {
    pub function: String,
    pub message: String,
}

impl LowerError {
    pub(crate) fn new(function: &str, message: impl Into<String>) -> Self {
        Self {
            function: function.into(),
            message: message.into(),
        }
    }
}

impl Display for LowerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "cannot lower function `{}`: {}",
            self.function, self.message
        )
    }
}

impl std::error::Error for LowerError {}
