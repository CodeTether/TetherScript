//! Live coverage for the `--grant-db` CLI capability.
//!
//! Proves the path a real deployment takes: a script run from the command line
//! reaching SQL through the `db` capability, with no Rust host embedding
//! tetherscript. Skipped unless `TETHERSCRIPT_PG_TEST_URL` is set, so the default
//! `cargo test` run stays hermetic. See docs/postgres-client.md for setup.

use std::process::Command;

/// Build the connection string the CLI flag expects.
fn grant() -> Option<String> {
    let target = std::env::var("TETHERSCRIPT_PG_TEST_URL").ok()?;
    Some(format!("postgres://tsuser:pencil@{target}/tsdb"))
}

/// Run a script, returning stdout and stderr.
fn run(source: &str, args: &[&str]) -> (String, String) {
    static CASE: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let case = CASE.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!("tether_grant_db_{}_{case}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join("case.tether");
    std::fs::write(&path, source).expect("write source");

    let mut command = Command::new(env!("CARGO_BIN_EXE_tetherscript"));
    command.arg("run");
    for arg in args {
        command.arg(arg);
    }
    let output = command
        .arg(&path)
        .output()
        .expect("tetherscript should run");
    (
        String::from_utf8_lossy(&output.stdout)
            .trim_end()
            .to_string(),
        String::from_utf8_lossy(&output.stderr)
            .trim_end()
            .to_string(),
    )
}

const QUERY: &str = r#"fn main() {
    let rows = db.query("SELECT 1 AS one", [])?
    println("rows: " + str(rows.len()))
}"#;

#[test]
fn a_granted_script_can_query() {
    let Some(grant) = grant() else { return };
    let (stdout, stderr) = run(QUERY, &["--grant-db", &grant]);
    assert!(
        stdout.contains("rows: 1"),
        "stdout: {stdout} stderr: {stderr}"
    );
}

/// The reference interpreter must agree with the VM, since it is the semantics
/// oracle and both install grants separately.
#[test]
fn the_interpreter_backend_also_receives_the_grant() {
    let Some(grant) = grant() else { return };
    let (stdout, stderr) = run(QUERY, &["--interp", "--grant-db", &grant]);
    assert!(
        stdout.contains("rows: 1"),
        "stdout: {stdout} stderr: {stderr}"
    );
}

/// Without the flag, `db` must be undefined rather than silently ambient.
#[test]
fn an_ungranted_script_fails_closed() {
    if grant().is_none() {
        return;
    }
    let (stdout, stderr) = run(QUERY, &[]);
    assert!(
        stderr.contains("undefined variable `db`"),
        "stdout: {stdout} stderr: {stderr}"
    );
}

/// A bound parameter must reach the server as data, never as SQL.
#[test]
fn a_bound_parameter_cannot_inject_sql() {
    let Some(grant) = grant() else { return };
    let source = r#"fn main() {
    let rows = db.query("SELECT $1 AS echoed", ["x'; DROP TABLE nothing; --"])?
    println("echoed: " + rows[0].echoed)
}"#;
    let (stdout, stderr) = run(source, &["--grant-db", &grant]);
    assert!(
        stdout.contains("DROP TABLE nothing"),
        "the hostile text must come back as a plain value: {stdout} {stderr}"
    );
}

/// A malformed URL must be rejected before any connection is attempted, so a
/// typo surfaces immediately rather than as a confusing query error later.
#[test]
fn a_malformed_grant_is_rejected_before_connecting() {
    let (_, stderr) = run(QUERY, &["--grant-db", "mysql://u:p@h/d"]);
    assert!(stderr.contains("postgres://"), "stderr: {stderr}");
}
