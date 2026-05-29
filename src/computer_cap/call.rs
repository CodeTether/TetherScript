//! Computer-use action envelope prepared from a tetherscript method call.

use crate::value::Value;

#[derive(Clone, Debug)]
pub(crate) struct ComputerCall {
    pub(crate) payload: Value,
    pub(crate) scope: &'static str,
}

impl ComputerCall {
    pub(crate) fn new(_action: &str, scope: &'static str, payload: Value) -> Self {
        Self { payload, scope }
    }
}
