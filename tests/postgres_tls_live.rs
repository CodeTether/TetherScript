//! Live coverage for PostgreSQL TLS negotiation.
//!
//! Requires a TLS-enabled server and the `openssl-tls` feature, so it is skipped
//! unless `TETHERSCRIPT_PG_TLS_URL` is set. See docs/postgres-client.md for the
//! container recipe.
//!
//! The assertions read `pg_stat_ssl` for the current backend rather than trusting
//! the client's own view: a client that believed it had TLS while sending cleartext
//! is exactly the failure this must catch.

use std::process::Command;

fn grant() -> Option<String> {
    std::env::var("TETHERSCRIPT_PG_TLS_URL").ok()
}

/// Run a script with the given `--grant-db` URL.
fn run(url: &str, source: &str) -> (String, String) {
    static CASE: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let case = CASE.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!("tether_pgtls_{}_{case}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join("case.tether");
    std::fs::write(&path, source).expect("write source");

    let output = Command::new(env!("CARGO_BIN_EXE_tetherscript"))
        .args(["run", "--grant-db", url])
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

/// Ask the backend whether *it* sees an encrypted connection.
const REPORT: &str = r#"fn main() {
    let rows = db.query("SELECT ssl FROM pg_stat_ssl WHERE pid = pg_backend_pid()", [])?
    println("encrypted: " + str(rows[0].ssl))
}"#;

#[test]
fn sslmode_require_produces_an_encrypted_connection() {
    let Some(url) = grant() else { return };
    let (stdout, stderr) = run(&format!("{url}?sslmode=require"), REPORT);
    assert!(
        stdout.contains("encrypted: true"),
        "the server must report TLS: {stdout} {stderr}"
    );
}

#[test]
fn sslmode_disable_stays_cleartext() {
    let Some(url) = grant() else { return };
    let (stdout, stderr) = run(&format!("{url}?sslmode=disable"), REPORT);
    assert!(
        stdout.contains("encrypted: false"),
        "disable must not negotiate TLS: {stdout} {stderr}"
    );
}

/// Absent an explicit mode, the connection must stay cleartext rather than
/// silently upgrading: the mode is the caller's decision to make.
#[test]
fn the_default_is_cleartext() {
    let Some(url) = grant() else { return };
    let (stdout, stderr) = run(&url, REPORT);
    assert!(
        stdout.contains("encrypted: false"),
        "no sslmode must mean no TLS: {stdout} {stderr}"
    );
}

/// A server that refuses TLS must fail the connection, never fall back.
#[test]
fn a_refused_upgrade_is_an_error_not_a_downgrade() {
    let Some(plain) = std::env::var("TETHERSCRIPT_PG_TEST_URL").ok() else {
        return;
    };
    let url = format!("postgres://tsuser:pencil@{plain}/tsdb?sslmode=require");
    let (stdout, stderr) = run(&url, REPORT);
    assert!(
        stderr.contains("refused TLS"),
        "must not silently downgrade: {stdout} {stderr}"
    );
    assert!(
        !stdout.contains("encrypted"),
        "no query may run after a refused upgrade: {stdout}"
    );
}
