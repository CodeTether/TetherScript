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
        if method != "lookup" {
            return Err(format!("db: unsupported method `{method}`"));
        }
        let Some(Value::Str(key)) = args.first() else {
            return Err("db.lookup: expected string key".into());
        };
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(Value::Str(Rc::new(format!("record:{key}"))))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
