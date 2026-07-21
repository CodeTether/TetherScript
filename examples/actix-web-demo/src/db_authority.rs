//! SQL-first tetherscript capability backed by the shared SQLx pool.

use std::any::Any;
use std::rc::Rc;

use tetherscript::capability::Authority;
use tetherscript::value::{Runtime, Value};
use tokio::runtime::Handle;

use crate::db_pool::DbPool;

pub struct DatabaseAuthority {
    pool: DbPool,
    runtime: Handle,
}

impl DatabaseAuthority {
    pub fn new(pool: DbPool, runtime: Handle) -> Self {
        Self { pool, runtime }
    }
}

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
            "query" => self
                .runtime
                .block_on(crate::db_query::query(&self.pool, arguments)),
            _ => Err(format!("db: unsupported method `{method}` (have: query)")),
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
