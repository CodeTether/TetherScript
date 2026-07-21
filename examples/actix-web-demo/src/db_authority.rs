//! SQLx implementation of tetherscript's host-neutral query contract.

use std::cell::RefCell;
use std::rc::Rc;

use tetherscript::database::QueryHandler;
use tetherscript::value::Value;
use tokio::runtime::Handle;

use crate::db_pool::DbPool;

pub(crate) struct SqlxQueryHandler {
    pool: DbPool,
    runtime: Handle,
}

impl SqlxQueryHandler {
    pub(crate) fn new(pool: DbPool, runtime: Handle) -> Self {
        Self { pool, runtime }
    }
}

impl QueryHandler for SqlxQueryHandler {
    fn query(&self, sql: &str, parameters: &[Value]) -> Result<Value, String> {
        let arguments = [
            Value::Str(Rc::new(sql.into())),
            Value::List(Rc::new(RefCell::new(parameters.to_vec()))),
        ];
        self.runtime
            .block_on(crate::db_query::query(&self.pool, &arguments))
    }
}
