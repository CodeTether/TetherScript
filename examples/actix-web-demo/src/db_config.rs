//! Remote PostgreSQL connection options shared by the server and bootstrap.

use sqlx::postgres::PgConnectOptions;

pub const DATABASE_NAME: &str = "tetherscript_actix_demo";

pub fn config(database: &str) -> Result<PgConnectOptions, Box<dyn std::error::Error>> {
    let host = std::env::var("DATABASE_HOST").unwrap_or_else(|_| "127.0.0.1".into());
    let port = std::env::var("DATABASE_PORT")
        .unwrap_or_else(|_| "5432".into())
        .parse()?;
    let user = std::env::var("DATABASE_USER").unwrap_or_else(|_| "postgres".into());
    let password = password()?;
    Ok(PgConnectOptions::new()
        .host(&host)
        .port(port)
        .username(&user)
        .password(&password)
        .database(database))
}

fn password() -> Result<String, Box<dyn std::error::Error>> {
    if let Ok(password) = std::env::var("PGPASSWORD") {
        return Ok(password);
    }
    let path = std::env::var("PGPASSWORD_FILE")
        .map_err(|_| "set PGPASSWORD or PGPASSWORD_FILE before connecting to the demo database")?;
    let password = std::fs::read_to_string(path)?;
    Ok(password.trim_end_matches(['\r', '\n']).to_string())
}
