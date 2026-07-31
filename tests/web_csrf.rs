//! Behaviour tests for the CSRF/state token built-ins.
//!
//! No expected token is hardcoded: every token here is minted by `csrf_token`,
//! and the forged ones are built by editing a real token, which is exactly how an
//! attacker would produce them. A signature value cannot be written down in
//! advance anyway, because each token carries a fresh random nonce.
//!
//! These drive the built-ins through the interpreter, since that is the surface
//! scripts actually see.

use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Tests run in parallel, so each case needs its own file name.
static CASE: AtomicUsize = AtomicUsize::new(0);

/// Run a tetherscript program from source text and return its trimmed stdout.
fn run(source: &str) -> String {
    let dir = std::env::temp_dir().join(format!("tether_csrf_{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir should be creatable");
    let path = dir.join(format!(
        "case_{}.tether",
        CASE.fetch_add(1, Ordering::SeqCst)
    ));
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
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

#[test]
fn token_then_verify_round_trips() {
    let out = run(r#"
fn main() {
    let token = csrf_token("shared-secret", 600).unwrap()
    println(str(csrf_verify(token, "shared-secret").unwrap()))
}
"#);
    assert_eq!(out, "true");
}

/// A different secret must not validate, and must be an error rather than false:
/// a bad signature means tampering, not an expired token.
#[test]
fn wrong_secret_is_a_bad_signature_error() {
    let out = run(r#"
fn main() {
    let token = csrf_token("shared-secret", 600).unwrap()
    let checked = csrf_verify(token, "other-secret")
    println(str(checked.is_err()))
    println(checked.err())
}
"#);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "true", "full output: {out}");
    assert!(
        lines[1].contains("bad signature"),
        "error should name the bad signature, got: {}",
        lines[1]
    );
}

/// An expired token is correctly signed, so it must verify as `false` rather than
/// erroring — the caller should restart the flow, not treat it as an attack.
///
/// A one-second TTL is used and waited out. That is slow but honest: the
/// alternative is forging a past `exp`, which requires signing a payload by hand
/// and would test my own re-implementation instead of the built-in.
#[test]
fn expired_token_verifies_as_false_not_an_error() {
    let out = run(r#"
fn main() {
    let token = csrf_token("shared-secret", 1).unwrap()
    println(str(csrf_verify(token, "shared-secret").unwrap()))
    sleep_ms(1600)
    let after = csrf_verify(token, "shared-secret")
    println(str(after.is_err()))
    println(str(after.unwrap()))
}
"#);
    assert_eq!(
        out, "true\nfalse\nfalse",
        "fresh token true; expired token is Ok(false), never an Err"
    );
}

#[test]
fn malformed_token_segment_count_is_named() {
    let out = run(r#"
fn main() {
    let bad = csrf_verify("only-one-segment", "shared-secret")
    println(str(bad.is_err()))
    println(bad.err())
}
"#);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "true", "full output: {out}");
    assert!(
        lines[1].contains("malformed token"),
        "error should say malformed token, got: {}",
        lines[1]
    );
}

/// Editing the payload invalidates the signature that covers it.
#[test]
fn tampered_payload_fails_the_signature_check() {
    let out = run(r#"
fn main() {
    let token = csrf_token("shared-secret", 600).unwrap()
    let parts = token.split(".")
    // Replace the payload with a different but well-formed base64url segment.
    let forged = "djEuZGVhZGJlZWYuMS4y" + "." + parts[1]
    let checked = csrf_verify(forged, "shared-secret")
    println(str(checked.is_err()))
    println(checked.err())
}
"#);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "true", "full output: {out}");
    assert!(
        lines[1].contains("bad signature"),
        "a re-encoded payload must fail the signature, got: {}",
        lines[1]
    );
}

/// The token travels in a URL query parameter, so it must carry no `=`, `+`, or `/`.
#[test]
fn token_has_no_padding_or_unsafe_characters() {
    let out = run(r#"
fn main() {
    let token = csrf_token("shared-secret", 600).unwrap()
    println(str(token.contains("=")))
    println(str(token.contains("+")))
    println(str(token.contains("/")))
}
"#);
    assert_eq!(out, "false\nfalse\nfalse");
}

/// A fresh nonce per token is what stops one token being guessed from another.
#[test]
fn successive_tokens_differ_because_of_the_nonce() {
    let out = run(r#"
fn main() {
    let a = csrf_token("shared-secret", 600).unwrap()
    let b = csrf_token("shared-secret", 600).unwrap()
    println(str(a == b))
    let nonce_a = csrf_claims(a).unwrap().nonce
    let nonce_b = csrf_claims(b).unwrap().nonce
    println(str(nonce_a == nonce_b))
}
"#);
    assert_eq!(out, "false\nfalse", "two tokens must not collide");
}

#[test]
fn claims_expose_nonce_iat_and_exp_without_verifying() {
    let out = run(r#"
fn main() {
    let token = csrf_token("shared-secret", 600).unwrap()
    let claims = csrf_claims(token).unwrap()
    println(str(claims.exp - claims.iat))
    println(str(claims.nonce.len()))
    // Deliberately readable with the wrong secret: claims does not authenticate.
    let other = csrf_claims(token).unwrap()
    println(str(other.exp == claims.exp))
}
"#);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "600", "exp must be iat + ttl");
    assert_eq!(lines[1], "32", "a 16-byte nonce is 32 hex characters");
    assert_eq!(lines[2], "true");
}

#[test]
fn non_positive_ttl_is_rejected() {
    let out = run(r#"
fn main() {
    let zero = csrf_token("shared-secret", 0)
    println(str(zero.is_err()))
    println(zero.err())
    println(str(csrf_token("shared-secret", -5).is_err()))
}
"#);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "true", "full output: {out}");
    assert!(
        lines[1].contains("bad ttl"),
        "error should name the bad ttl, got: {}",
        lines[1]
    );
    assert_eq!(lines[2], "true");
}

/// A segment outside the URL-safe alphabet must be reported as bad base64url.
#[test]
fn bad_base64url_is_named() {
    let out = run(r#"
fn main() {
    let bad = csrf_verify("not+valid.also/bad", "shared-secret")
    println(str(bad.is_err()))
    println(bad.err())
}
"#);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "true", "full output: {out}");
    assert!(
        lines[1].contains("base64url"),
        "error should mention base64url, got: {}",
        lines[1]
    );
}
