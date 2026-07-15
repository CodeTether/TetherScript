use crate::ir::{Instruction, Operation, Parameter, ValueId};

use super::{scope::Scope, LowerError};

pub(super) struct Builder {
    pub(super) function: String,
    pub(super) params: Vec<Parameter>,
    pub(super) instructions: Vec<Instruction>,
    scope: Scope,
    next_value: u32,
}

impl Builder {
    pub(super) fn new(function: &str, names: &[String]) -> Self {
        let mut builder = Self {
            function: function.into(),
            params: vec![],
            instructions: vec![],
            scope: Scope::new(function),
            next_value: 0,
        };
        for name in names {
            builder.add_parameter(name);
        }
        builder
    }

    fn add_parameter(&mut self, name: &str) {
        let value = self.allocate();
        self.scope.bind(name, value);
        self.params.push(Parameter {
            name: name.into(),
            value,
        });
    }

    fn allocate(&mut self) -> ValueId {
        let value = ValueId(self.next_value);
        self.next_value += 1;
        value
    }

    pub(super) fn emit(&mut self, operation: Operation) -> ValueId {
        let result = self.allocate();
        self.instructions.push(Instruction { result, operation });
        result
    }

    pub(super) fn bind(&mut self, name: &str, value: ValueId) {
        self.scope.bind(name, value);
    }

    pub(super) fn resolve(&self, name: &str) -> Result<ValueId, LowerError> {
        self.scope.resolve(name)
    }
}
