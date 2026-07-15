use std::fmt::{self, Display};

/// Describes a structural violation in Tether IR.
///
/// # Examples
///
/// ```
/// use tetherscript::ir::{verify, Function, Module};
/// let module = Module { functions: vec![Function {
///     name: "broken".into(), params: vec![], blocks: vec![],
/// }] };
/// assert!(verify(&module).is_err());
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerifyError {
    pub function: Option<String>,
    pub message: String,
}

impl VerifyError {
    pub(super) fn module(message: impl Into<String>) -> Self {
        Self {
            function: None,
            message: message.into(),
        }
    }

    pub(super) fn function(name: &str, message: impl Into<String>) -> Self {
        Self {
            function: Some(name.into()),
            message: message.into(),
        }
    }
}

impl Display for VerifyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.function {
            Some(name) => write!(formatter, "invalid function `{name}`: {}", self.message),
            None => write!(formatter, "invalid module: {}", self.message),
        }
    }
}

impl std::error::Error for VerifyError {}
