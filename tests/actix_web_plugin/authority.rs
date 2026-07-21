use std::any::Any;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use tetherscript::capability::Authority;
use tetherscript::value::{Runtime, Value};

pub(super) struct DbAuthority {
    calls: Arc<AtomicUsize>,
}

impl DbAuthority {
    pub fn new(calls: Arc<AtomicUsize>) -> Self {
        Self { calls }
    }
}

impl Authority for DbAuthority {
    fn narrow(&self, _params: &Value) -> Result<Rc<dyn Authority>, String> {
        Ok(Rc::new(Self::new(self.calls.clone())))
    }

    fn invoke(
        &self,
        _runtime: &mut dyn Runtime,
        method: &str,
        args: &[Value],
    ) -> Result<Value, String> {
        if method != "query" {
            return Err(format!("db: unsupported method `{method}`"));
        }
        let [Value::Str(sql), Value::List(params)] = args else {
            return Err("db.query: expected SQL and parameters".into());
        };
        if sql.as_str() != "SELECT value FROM records WHERE id = $1" {
            return Err(format!("db.query: unexpected SQL `{sql}`"));
        }
        let params = params.borrow();
        let Some(Value::Str(key)) = params.first() else {
            return Err("db.query: expected string parameter".into());
        };
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(super::db_value::row("value", format!("record:{key}")))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
