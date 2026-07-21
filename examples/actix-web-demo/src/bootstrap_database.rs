//! Create the demo database without touching any other PostgreSQL database.

use sqlx::postgres::PgConnection;
use sqlx::Connection;

use crate::db_config::{self, DATABASE_NAME};

pub(crate) async fn ensure() -> Result<(), Box<dyn std::error::Error>> {
    let options = db_config::config("postgres")?;
    let mut admin = PgConnection::connect_with(&options).await?;
    let exists: bool =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM pg_database WHERE datname = $1)")
            .bind(DATABASE_NAME)
            .fetch_one(&mut admin)
            .await?;
    if exists {
        println!("database {DATABASE_NAME} already exists");
        return Ok(());
    }
    sqlx::query("CREATE DATABASE tetherscript_actix_demo")
        .execute(&mut admin)
        .await?;
    println!("created database {DATABASE_NAME}");
    Ok(())
}
