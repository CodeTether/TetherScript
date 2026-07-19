//! Tetherscript capability backed by the shared PostgreSQL pool.

use std::any::Any;
use std::rc::Rc;

use tetherscript::capability::Authority;
use tetherscript::value::{Runtime, Value};

use crate::db_pool::DbPool;

pub struct DatabaseAuthority {
    pool: DbPool,
}

impl DatabaseAuthority {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

impl Authority for DatabaseAuthority {
    fn narrow(&self, _params: &Value) -> Result<Rc<dyn Authority>, String> {
        Ok(Rc::new(Self::new(self.pool.clone())))
    }

    fn invoke(
        &self,
        _runtime: &mut dyn Runtime,
        method: &str,
        arguments: &[Value],
    ) -> Result<Value, String> {
        crate::db_query::invoke(&self.pool, method, arguments)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
