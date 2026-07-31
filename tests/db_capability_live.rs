//! Live coverage for the `db` capability over the native PostgreSQL client.
//!
//! Proves the whole path a script actually takes: PluginHost grant, the
//! DatabaseAuthority contract, extended-protocol parameter binding, and row
//! decoding. Skipped unless `TETHERSCRIPT_PG_TEST_URL` is set, so the default
//! `cargo test` run stays hermetic. See docs/postgres-client.md for setup.

use std::rc::Rc;

use tetherscript::database::DatabaseAuthority;
use tetherscript::plugin::PluginHost;
use tetherscript::postgres::{Config, PostgresHandler};
use tetherscript::value::Value;

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

/// Grant the native client as `db` and load the example script.
fn host() -> Option<PluginHost> {
    let handler = PostgresHandler::connect(&config()?).expect("connect should succeed");
    let mut host = PluginHost::new();
    host.grant("db", Rc::new(DatabaseAuthority::new(handler)));
    Some(host)
}

#[test]
fn script_reads_rows_through_the_db_capability() {
    let Some(host) = host() else { return };
    let mut script = host
        .load_file("examples/db_capability.tether")
        .expect("example should load");
    let outcome = script.call("report", &[]).expect("report should succeed");
    assert!(
        outcome.stdout.contains("user: Riley"),
        "stdout: {}",
        outcome.stdout
    );
    assert!(
        outcome.stdout.contains("rows: 2"),
        "stdout: {}",
        outcome.stdout
    );
}

/// A bound parameter must be data, never SQL. This is the reason Parse/Bind
/// exists, so it is asserted rather than assumed.
#[test]
fn bound_parameters_cannot_inject_sql() {
    let Some(host) = host() else { return };
    let mut script = host
        .load_file("examples/db_capability.tether")
        .expect("example should load");
    let outcome = script
        .call("injection_is_inert", &[])
        .expect("hostile parameter must not error");
    // Value is Ok(count); a successful injection would have changed the count or
    // dropped the table, and the following read proves the table survived.
    assert_eq!(format!("{:?}", outcome.value), "Ok(0)");

    let mut again = host
        .load_file("examples/db_capability.tether")
        .expect("example should reload");
    let after = again
        .call("report", &[])
        .expect("table must still exist after the hostile parameter");
    assert!(after.stdout.contains("rows: 2"), "stdout: {}", after.stdout);
}

/// Parameters bind positionally by `$n`.
#[test]
fn script_binds_a_parameter_by_position() {
    let Some(host) = host() else { return };
    let mut script = host
        .load_file("examples/db_capability.tether")
        .expect("example should load");
    let outcome = script
        .call("find_user", &[Value::Int(2)])
        .expect("find_user should succeed");
    assert!(
        format!("{:?}", outcome.value).contains("Ada"),
        "value: {:?}",
        outcome.value
    );
}

/// A missing row must surface the script's own Err, not a transport failure.
#[test]
fn script_error_path_reports_a_missing_row() {
    let Some(host) = host() else { return };
    let mut script = host
        .load_file("examples/db_capability.tether")
        .expect("example should load");
    let outcome = script
        .call("find_user", &[Value::Int(9999)])
        .expect("call itself should not fail");
    assert!(
        format!("{:?}", outcome.value).contains("no user with id 9999"),
        "value: {:?}",
        outcome.value
    );
}
