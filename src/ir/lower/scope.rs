use std::collections::HashMap;

use crate::ir::ValueId;

use super::LowerError;

pub(super) struct Scope {
    function: String,
    names: HashMap<String, ValueId>,
}

impl Scope {
    pub(super) fn new(function: &str) -> Self {
        Self {
            function: function.into(),
            names: HashMap::new(),
        }
    }

    pub(super) fn bind(&mut self, name: &str, value: ValueId) {
        self.names.insert(name.into(), value);
    }

    pub(super) fn resolve(&self, name: &str) -> Result<ValueId, LowerError> {
        self.names
            .get(name)
            .copied()
            .ok_or_else(|| LowerError::new(&self.function, format!("unknown value `{name}`")))
    }
}
