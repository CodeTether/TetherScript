//! Shared asynchronous SQLx PostgreSQL connection pool.

use sqlx::postgres::PgPoolOptions;

use crate::db_config::{self, DATABASE_NAME};

pub type DbPool = sqlx::PgPool;

pub async fn create() -> Result<DbPool, Box<dyn std::error::Error>> {
    let options = db_config::config(DATABASE_NAME)?;
    Ok(PgPoolOptions::new()
        .max_connections(16)
        .connect_with(options)
        .await?)
}
