//! End-to-end coverage for bitwise operators on both backends.
//!
//! `&` is bitwise AND in infix position and a borrow in prefix position, so these
//! assert both readings in the same program. Every case runs under the VM and the
//! tree-walking interpreter, because the two must agree on semantics.

use std::process::Command;

/// Run `source` on `backend`, asserting success and returning trimmed stdout.
fn stdout_on(source: &str, backend: &[&str]) -> String {
    static CASE: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let case = CASE.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!("tether_bitwise_{}_{case}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join("case.tether");
    std::fs::write(&path, source).expect("write source");

    let output = Command::new(env!("CARGO_BIN_EXE_tetherscript"))
        .arg("run")
        .args(backend)
        .arg(&path)
        .output()
        .expect("tetherscript should run");
    assert!(
        output.status.success(),
        "script failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout)
        .trim_end()
        .to_string()
}

/// Assert an expression prints `expected` under both backends.
fn both_backends(expression: &str, expected: &str) {
    let source = format!("fn main() {{ println({expression}) }}");
    assert_eq!(stdout_on(&source, &[]), expected, "vm: {expression}");
    assert_eq!(
        stdout_on(&source, &["--interp"]),
        expected,
        "interp: {expression}"
    );
}

#[test]
fn and_or_xor_produce_rust_results() {
    both_backends("12 & 10", "8");
    both_backends("12 | 10", "14");
    both_backends("12 ^ 10", "6");
}

#[test]
fn shifts_produce_rust_results() {
    both_backends("1 << 4", "16");
    both_backends("256 >> 4", "16");
}

#[test]
fn right_shift_is_arithmetic_and_preserves_sign() {
    both_backends("-8 >> 1", "-4");
    both_backends("-1 >> 32", "-1");
}

#[test]
fn bitwise_not_complements_every_bit() {
    both_backends("~0", "-1");
    both_backends("~5", "-6");
}

#[test]
fn precedence_matches_rust() {
    // `|` looser than `^` looser than `&` looser than shifts, all tighter than
    // comparison. Cross-checked against Rust and Python, which agree.
    both_backends("1 | 2 ^ 3 & 4", "3");
    both_backends("1 << 2 + 3", "32");
    both_backends("1 & 3 == 1", "true");
}

#[test]
fn prefix_borrow_still_works_alongside_infix_and() {
    let source = "fn main() { let xs = [1, 2, 3]  let r = &xs  println(r.len() & 3) }";

    assert_eq!(stdout_on(source, &[]), "3");
    assert_eq!(stdout_on(source, &["--interp"]), "3");
}

#[test]
fn logical_and_is_unaffected() {
    both_backends("true && false", "false");
    both_backends("true || false", "true");
}

/// Bitwise operators reject bools on purpose.
///
/// `a & b` where both sides are bool is nearly always a mistyped `&&`. Rust
/// accepts it as non-short-circuiting logical and; tetherscript refuses, because
/// silently accepting the typo is how a short-circuit bug ships.
#[test]
fn bitwise_and_on_bools_is_rejected() {
    let dir = std::env::temp_dir().join(format!("tether_bool_and_{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join("case.tether");
    std::fs::write(&path, "fn main() { println(true & false) }").expect("write source");

    let output = Command::new(env!("CARGO_BIN_EXE_tetherscript"))
        .arg("run")
        .arg(&path)
        .output()
        .expect("tetherscript should run");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success(), "bool & bool should fail");
    assert!(stderr.contains("BitAnd"), "got: {stderr}");
    assert!(stderr.contains("bool"), "got: {stderr}");
}

#[test]
fn an_oversized_shift_count_is_a_named_error() {
    let dir = std::env::temp_dir().join(format!("tether_shift_err_{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join("case.tether");
    std::fs::write(&path, "fn main() { println(1 << 64) }").expect("write source");

    let output = Command::new(env!("CARGO_BIN_EXE_tetherscript"))
        .arg("run")
        .arg(&path)
        .output()
        .expect("tetherscript should run");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success(), "an oversized shift should fail");
    assert!(stderr.contains("shift count"), "got: {stderr}");
    assert!(stderr.contains("64"), "got: {stderr}");
}
