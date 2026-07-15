use std::collections::HashSet;

use crate::ir::ValueId;

use super::VerifyError;

pub(super) struct Definitions<'a> {
    function: &'a str,
    values: HashSet<ValueId>,
}

impl<'a> Definitions<'a> {
    pub(super) fn new(function: &'a str) -> Self {
        Self {
            function,
            values: HashSet::new(),
        }
    }

    pub(super) fn define(&mut self, value: ValueId) -> Result<(), VerifyError> {
        if self.values.insert(value) {
            Ok(())
        } else {
            Err(VerifyError::function(
                self.function,
                format!("value %{} is defined more than once", value.0),
            ))
        }
    }

    pub(super) fn require(&self, value: ValueId) -> Result<(), VerifyError> {
        if self.values.contains(&value) {
            Ok(())
        } else {
            Err(VerifyError::function(
                self.function,
                format!("value %{} is used before definition", value.0),
            ))
        }
    }
}
