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
            "begin" | "commit" | "rollback" | "pool_size" => {
                super::unit_call::call_unit(self.handler.as_ref(), method, arguments)
            }
            _ => Err(format!(
                "db: unsupported method `{method}` (have: query, begin, commit, rollback, pool_size)"
            )),
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
