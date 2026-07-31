//! Behaviour tests for the HMAC/hex built-ins.
//!
//! Expected MAC values are the published RFC 4231 vectors for HMAC-SHA-256; none
//! of them were invented. Where a case has no standard vector (arbitrary input to
//! `hex_encode`, for example), the expectation is derived from the hex alphabet
//! itself rather than from a guessed digest.
//!
//! These drive the built-ins through the interpreter, since that is the surface
//! scripts actually see.

use std::process::Command;

/// Run a tetherscript program from source text and return its stdout.
fn run(source: &str) -> String {
    let dir = std::env::temp_dir().join(format!(
        "tether_web_hmac_{}_{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    std::fs::create_dir_all(&dir).expect("temp dir should be creatable");
    let path = dir.join("case.tether");
    std::fs::write(&path, source).expect("source should be writable");

    let output = Command::new(env!("CARGO_BIN_EXE_tetherscript"))
        .arg("run")
        .arg(&path)
        .output()
        .expect("tetherscript should run");
    assert!(
        output.status.success(),
        "script failed\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout)
        .trim_end()
        .to_string()
}

/// RFC 4231 test case 2: key "Jefe", data "what do ya want for nothing?".
#[test]
fn hmac_sha256_hex_matches_rfc4231_case_2() {
    let out =
        run(r#"fn main() { println(hmac_sha256_hex("Jefe", "what do ya want for nothing?")) }"#);
    assert_eq!(
        out,
        "5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843"
    );
}

#[test]
fn hmac_sha256_hex_matches_rfc4231_case_1() {
    let out = run(r#"fn main() {
    let key = hex_decode("0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b").unwrap()
    println(hmac_sha256_hex(key, "Hi There"))
}"#);
    assert_eq!(
        out,
        "b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7"
    );
}

#[test]
fn hmac_sha256_hex_changes_with_the_key() {
    let out = run(r#"fn main() {
    let a = hmac_sha256_hex("key-one", "same message")
    let b = hmac_sha256_hex("key-two", "same message")
    println(str(a == b))
}"#);
    assert_eq!(out, "false");
}

#[test]
fn hex_encode_uses_the_lowercase_alphabet() {
    // "0" is 0x30 and "\n" is 0x0a; both come straight from ASCII.
    let out = run(r#"fn main() { println(hex_encode("0")) }"#);
    assert_eq!(out, "30");
}

#[test]
fn hex_encode_round_trips_through_hex_decode() {
    let out = run(r#"fn main() {
    let encoded = hex_encode("hello, tetherscript")
    println(hex_decode(encoded).unwrap())
}"#);
    assert_eq!(out, "hello, tetherscript");
}

#[test]
fn hex_decode_accepts_uppercase_input() {
    let out = run(r#"fn main() { println(hex_decode("4A4B").unwrap()) }"#);
    assert_eq!(out, "JK");
}

#[test]
fn hex_decode_rejects_an_odd_length_by_naming_it() {
    let out = run(r#"fn main() {
    let r = hex_decode("abc")
    println(str(r.is_err()))
    println(r.err())
}"#);
    assert!(out.starts_with("true"), "got: {out}");
    assert!(
        out.contains("odd length 3"),
        "error should name the length: {out}"
    );
}

#[test]
fn hex_decode_names_the_offending_character() {
    let out = run(r#"fn main() {
    let r = hex_decode("4z")
    println(str(r.is_err()))
    println(r.err())
}"#);
    assert!(out.starts_with("true"), "got: {out}");
    assert!(
        out.contains('z'),
        "error should name the bad character: {out}"
    );
    assert!(
        out.contains("position 1"),
        "error should name the position: {out}"
    );
}

#[test]
fn constant_time_eq_reports_equality() {
    let out = run(r#"fn main() {
    println(str(constant_time_eq("abc", "abc")))
    println(str(constant_time_eq("abc", "abd")))
    println(str(constant_time_eq("abc", "ab")))
    println(str(constant_time_eq("", "")))
}"#);
    assert_eq!(out, "true\nfalse\nfalse\ntrue");
}

/// A verifier is expected to compare a computed MAC against a supplied one.
#[test]
fn constant_time_eq_accepts_a_recomputed_mac() {
    let out = run(r#"fn main() {
    let expected = hmac_sha256_hex("secret", "payload")
    println(str(constant_time_eq(hmac_sha256_hex("secret", "payload"), expected)))
    println(str(constant_time_eq(hmac_sha256_hex("wrong", "payload"), expected)))
}"#);
    assert_eq!(out, "true\nfalse");
}

#[test]
fn builtins_reject_non_string_arguments_by_name() {
    let dir = std::env::temp_dir().join(format!("tether_web_hmac_type_{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir should be creatable");
    let path = dir.join("bad.tether");
    std::fs::write(&path, "fn main() { println(hex_encode(7)) }\n").expect("write");
    let run = Command::new(env!("CARGO_BIN_EXE_tetherscript"))
        .arg("run")
        .arg(&path)
        .output()
        .expect("tetherscript should run");
    let stderr = String::from_utf8_lossy(&run.stderr);
    assert!(!run.status.success(), "passing an int must fail");
    assert!(
        stderr.contains("hex_encode: input") && stderr.contains("int"),
        "error should name the argument and the type, got: {stderr}"
    );
}
