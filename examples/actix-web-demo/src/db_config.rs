//! Remote PostgreSQL connection configuration.

use postgres::Config;

pub const DATABASE_NAME: &str = "tetherscript_actix_demo";

pub fn config(database: &str) -> Result<Config, Box<dyn std::error::Error>> {
    let host = std::env::var("DATABASE_HOST").unwrap_or_else(|_| "127.0.0.1".into());
    let port = std::env::var("DATABASE_PORT")
        .unwrap_or_else(|_| "5432".into())
        .parse()?;
    let user = std::env::var("DATABASE_USER").unwrap_or_else(|_| "postgres".into());
    let password = password()?;
    let mut config = Config::new();
    config
        .host(&host)
        .port(port)
        .user(&user)
        .password(password)
        .dbname(database);
    Ok(config)
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
