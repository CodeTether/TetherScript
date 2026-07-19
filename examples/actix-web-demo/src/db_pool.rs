//! Shared r2d2 PostgreSQL connection pool.

use postgres::NoTls;
use r2d2::Pool;
use r2d2_postgres::PostgresConnectionManager;

use crate::db_config::{self, DATABASE_NAME};

pub type DbPool = Pool<PostgresConnectionManager<NoTls>>;

pub fn create() -> Result<DbPool, Box<dyn std::error::Error>> {
    let manager = PostgresConnectionManager::new(db_config::config(DATABASE_NAME)?, NoTls);
    Ok(Pool::builder().max_size(16).build(manager)?)
}
