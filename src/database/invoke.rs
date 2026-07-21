use std::any::Any;
use std::rc::Rc;

use crate::capability::Authority;
use crate::value::{Runtime, Value};

use super::DatabaseAuthority;

impl Authority for DatabaseAuthority {
    fn narrow(&self, _params: &Value) -> Result<Rc<dyn Authority>, String> {
        Err("db: SQL authority does not support narrowing".into())
    }

    fn invoke(
        &self,
        _runtime: &mut dyn Runtime,
        method: &str,
        arguments: &[Value],
    ) -> Result<Value, String> {
        match method {
            "query" => super::query::call(self.handler.as_ref(), arguments),
            _ => Err(format!("db: unsupported method `{method}` (have: query)")),
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
