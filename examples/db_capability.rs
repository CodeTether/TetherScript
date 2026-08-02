//! Rust host that grants the `db` capability to a `.tether` script.
//!
//! Connects with the native PostgreSQL client, wraps it in a
//! [`DatabaseAuthority`], and grants it as `db` so
//! `examples/db_capability.tether` can run queries. No database driver is
//! involved: the wire protocol is implemented in-tree.
//!
//! Run against a throwaway server:
//!
//! ```text
//! docker run -d --rm --name ts_pg_test -e POSTGRES_PASSWORD=pencil \
//!   -e POSTGRES_USER=tsuser -e POSTGRES_DB=tsdb -p 55432:5432 postgres:16
//! docker exec ts_pg_test psql -U tsuser -d tsdb -c \
//!   "CREATE TABLE users (id int, name text, active bool, note text);
//!    INSERT INTO users VALUES (1,'Riley',true,NULL),(2,'Ada',false,'hi');"
//!
//! TETHERSCRIPT_PG_URL=127.0.0.1:55432 cargo run --example db_capability
//! ```

use std::rc::Rc;

use tetherscript::database::DatabaseAuthority;
use tetherscript::plugin::PluginHost;
use tetherscript::postgres::{Config, PostgresHandler};

fn main() -> Result<(), String> {
    // The host decides where credentials come from; the client never reads them
    // from the environment itself.
    let target =
        std::env::var("TETHERSCRIPT_PG_URL").unwrap_or_else(|_| "127.0.0.1:5432".to_string());
    let (host_name, port) = target
        .split_once(':')
        .ok_or("TETHERSCRIPT_PG_URL must look like host:port")?;

    let handler = PostgresHandler::connect(&Config {
        host: host_name.to_string(),
        port: port.parse().map_err(|_| "port must be a number")?,
        user: std::env::var("TETHERSCRIPT_PG_USER").unwrap_or_else(|_| "tsuser".into()),
        password: std::env::var("TETHERSCRIPT_PG_PASSWORD").unwrap_or_else(|_| "pencil".into()),
        database: std::env::var("TETHERSCRIPT_PG_DB").unwrap_or_else(|_| "tsdb".into()),
        // Cleartext for a local throwaway container; production needs TLS.
        tls: false,
    })?;

    let mut host = PluginHost::new();
    host.grant("db", Rc::new(DatabaseAuthority::new(handler)));

    let mut script = host
        .load_file("examples/db_capability.tether")
        .map_err(|error| error.to_string())?;
    let outcome = script
        .call("report", &[])
        .map_err(|error| error.to_string())?;
    // PluginHost captures script output rather than writing it through directly.
    print!("{}", outcome.stdout);
    println!("report returned: {:?}", outcome.value);
    Ok(())
}
