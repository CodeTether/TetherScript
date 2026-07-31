//! Live integration coverage for the native PostgreSQL client.
//!
//! These tests need a real server, because the whole point of the client is wire
//! compatibility: the SCRAM exchange, message framing, and row decoding cannot be
//! proven against a mock. They are skipped unless `TETHERSCRIPT_PG_TEST_URL` is
//! set, so the default `cargo test` run stays hermetic.
//!
//! ```text
//! docker run -d --rm --name ts_pg_test -e POSTGRES_PASSWORD=pencil \
//!   -e POSTGRES_USER=tsuser -e POSTGRES_DB=tsdb -p 55432:5432 postgres:16
//! TETHERSCRIPT_PG_TEST_URL=127.0.0.1:55432 cargo test --test postgres_live
//! ```

use tetherscript::postgres::{Config, Connection};
use tetherscript::value::Value;

/// Resolve the test server address, or `None` when live testing is not enabled.
fn config() -> Option<Config> {
    let target = std::env::var("TETHERSCRIPT_PG_TEST_URL").ok()?;
    let (host, port) = target.split_once(':')?;
    Some(Config {
        host: host.to_string(),
        port: port.parse().ok()?,
        user: "tsuser".into(),
        password: "pencil".into(),
        database: "tsdb".into(),
    })
}

fn rows(value: &Value) -> Vec<Value> {
    match value {
        Value::List(items) => items.borrow().clone(),
        other => panic!("expected a row list, got {}", other.type_name()),
    }
}

fn field(row: &Value, name: &str) -> Value {
    match row {
        Value::Map(map) => map
            .borrow()
            .get(name)
            .cloned()
            .unwrap_or_else(|| panic!("row has no column `{name}`")),
        other => panic!("expected a row map, got {}", other.type_name()),
    }
}

/// The SCRAM-SHA-256 handshake must succeed against a real server.
#[test]
fn connects_with_scram_authentication() {
    let Some(config) = config() else { return };
    let mut connection = Connection::connect(&config).expect("SCRAM connect should succeed");
    let result = connection
        .simple_query("SELECT 1 AS one")
        .expect("query should succeed");
    let rows = rows(&result);
    assert_eq!(rows.len(), 1);
    assert_eq!(field(&rows[0], "one"), Value::Int(1));
}

#[test]
fn decodes_columns_types_and_nulls() {
    let Some(config) = config() else { return };
    let mut connection = Connection::connect(&config).expect("connect should succeed");
    let result = connection
        .simple_query("SELECT id, name, active, score, note FROM users ORDER BY id")
        .expect("query should succeed");
    let rows = rows(&result);
    assert_eq!(rows.len(), 2);

    assert_eq!(field(&rows[0], "id"), Value::Int(1));
    assert_eq!(field(&rows[0], "active"), Value::Bool(true));
    assert_eq!(field(&rows[0], "score"), Value::Float(9.5));
    // SQL NULL decodes to nil, not an empty string.
    assert_eq!(field(&rows[0], "note"), Value::Nil);

    assert_eq!(field(&rows[1], "active"), Value::Bool(false));
    match field(&rows[1], "name") {
        Value::Str(name) => assert_eq!(name.as_str(), "Ada"),
        other => panic!("name should be a str, got {}", other.type_name()),
    }
}

/// A SQL error must surface the server's message, not a transport failure.
#[test]
fn reports_server_error_with_sqlstate() {
    let Some(config) = config() else { return };
    let mut connection = Connection::connect(&config).expect("connect should succeed");
    let error = connection
        .simple_query("SELECT * FROM no_such_table")
        .expect_err("missing table must error");
    assert!(error.contains("no_such_table"), "got: {error}");
    assert!(error.contains("SQLSTATE 42P01"), "got: {error}");
}

/// The connection must stay usable after a failed query.
#[test]
fn connection_is_reusable_after_an_error() {
    let Some(config) = config() else { return };
    let mut connection = Connection::connect(&config).expect("connect should succeed");
    let _ = connection.simple_query("SELECT * FROM no_such_table");
    let result = connection
        .simple_query("SELECT 42 AS answer")
        .expect("connection should still work after an error");
    assert_eq!(field(&rows(&result)[0], "answer"), Value::Int(42));
}

#[test]
fn rejects_a_wrong_password() {
    let Some(mut config) = config() else { return };
    config.password = "not-the-password".into();
    let error = Connection::connect(&config).expect_err("wrong password must fail");
    assert!(
        error.contains("postgres:"),
        "error should be qualified, got: {error}"
    );
}

/// Extended-protocol binding must return the same rows as inline SQL would.
#[test]
fn binds_parameters_through_the_extended_protocol() {
    let Some(config) = config() else { return };
    let mut connection = Connection::connect(&config).expect("connect should succeed");
    let result = connection
        .query("SELECT name FROM users WHERE id = $1", &[Value::Int(2)])
        .expect("parameterized query should succeed");
    let rows = rows(&result);
    assert_eq!(rows.len(), 1);
    match field(&rows[0], "name") {
        Value::Str(name) => assert_eq!(name.as_str(), "Ada"),
        other => panic!("expected str, got {}", other.type_name()),
    }
}

/// A bound value is data, never SQL. This is the whole point of Parse/Bind.
#[test]
fn a_bound_parameter_cannot_terminate_the_statement() {
    let Some(config) = config() else { return };
    let mut connection = Connection::connect(&config).expect("connect should succeed");
    let hostile = Value::Str(std::rc::Rc::new("Ada'; DROP TABLE users; --".into()));
    let result = connection
        .query("SELECT id FROM users WHERE name = $1", &[hostile])
        .expect("hostile input must be treated as a plain value");
    assert_eq!(rows(&result).len(), 0, "no name matches that literal");

    // The table must still be there.
    let survived = connection
        .simple_query("SELECT id FROM users")
        .expect("users table must still exist");
    assert_eq!(rows(&survived).len(), 2);
}

#[test]
fn binds_nil_as_sql_null() {
    let Some(config) = config() else { return };
    let mut connection = Connection::connect(&config).expect("connect should succeed");
    let result = connection
        .query(
            "SELECT id FROM users WHERE note IS NOT DISTINCT FROM $1",
            &[Value::Nil],
        )
        .expect("nil should bind as NULL");
    assert_eq!(rows(&result).len(), 1, "exactly one row has a NULL note");
}

#[test]
fn multiple_parameters_bind_positionally() {
    let Some(config) = config() else { return };
    let mut connection = Connection::connect(&config).expect("connect should succeed");
    let result = connection
        .query(
            "SELECT id FROM users WHERE id = $1 AND active = $2",
            &[Value::Int(1), Value::Bool(true)],
        )
        .expect("two parameters should bind");
    assert_eq!(rows(&result).len(), 1);
}
