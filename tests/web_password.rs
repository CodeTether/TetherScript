//! Behaviour tests for the password hashing built-ins.
//!
//! These drive the built-ins through the interpreter, since that is the surface a
//! script actually sees.
//!
//! # On expected values
//!
//! Only one test asserts a fixed digest, and it is **not** invented: it is the
//! published PBKDF2-HMAC-SHA-256 vector for password `"password"`, salt `"salt"`,
//! 4096 iterations, 32-byte output — hex `c5e478d5…aa98134a` — which is already
//! pinned independently in `src/postgres/hmac_tests.rs`
//! (`pbkdf2_matches_rfc6070_style_sha256_vector`). Feeding it in as a stored PHC
//! string proves this module's PBKDF2, salt handling, and base64 all agree with
//! that verified implementation.
//!
//! Every other test asserts a *property* — round-tripping, difference, rejection —
//! rather than a hash value, so nothing here depends on a guessed digest.

use std::process::Command;

/// Run a tetherscript program from source text and return its stdout.
fn run(source: &str) -> String {
    let output = spawn(source);
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

/// Run a program without asserting success, for the malformed-input cases.
fn spawn(source: &str) -> std::process::Output {
    // A monotonic counter, not the thread id: the test harness reuses threads, so
    // two cases can land on the same id and overwrite each other's source file.
    static CASE: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let case = CASE.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let dir =
        std::env::temp_dir().join(format!("tether_web_password_{}_{case}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir should be creatable");
    let path = dir.join("case.tether");
    std::fs::write(&path, source).expect("source should be writable");
    Command::new(env!("CARGO_BIN_EXE_tetherscript"))
        .arg("run")
        .arg(&path)
        .output()
        .expect("tetherscript should run")
}

#[test]
fn hash_then_verify_round_trips() {
    let out = run(r#"fn main() {
    let stored = password_hash("correct horse battery staple").unwrap()
    println(str(password_verify("correct horse battery staple", stored).unwrap()))
}"#);
    assert_eq!(out, "true");
}

#[test]
fn a_wrong_password_does_not_verify() {
    let out = run(r#"fn main() {
    let stored = password_hash("right").unwrap()
    println(str(password_verify("wrong", stored).unwrap()))
}"#);
    assert_eq!(out, "false");
}

/// The salt must be random, so the same password must not hash identically.
/// Without this, one precomputed table would break every account at once.
#[test]
fn two_hashes_of_the_same_password_differ() {
    let out = run(r#"fn main() {
    let a = password_hash("same password").unwrap()
    let b = password_hash("same password").unwrap()
    println(str(a == b))
    println(str(password_verify("same password", a).unwrap()))
    println(str(password_verify("same password", b).unwrap()))
}"#);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "false", "salt must differ per call: {out}");
    // Both must still verify, proving each carries its own working salt.
    assert_eq!(lines[1], "true", "full output: {out}");
    assert_eq!(lines[2], "true", "full output: {out}");
}

#[test]
fn the_encoded_form_is_self_describing() {
    let out = run(r#"fn main() {
    let stored = password_hash("pw").unwrap()
    println(str(stored.starts_with("$pbkdf2-sha256$i=")))
    println(str(stored.split("$").len()))
}"#);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "true", "full output: {out}");
    // Leading `$` yields an empty first field, so five parts in total.
    assert_eq!(lines[1], "5", "full output: {out}");
}

/// The default cost must meet current guidance, not a token value.
#[test]
fn default_cost_is_at_least_600000() {
    let out = run(r#"fn main() {
    let stored = password_hash("pw").unwrap()
    println(str(password_needs_rehash(stored, 600000).unwrap()))
}"#);
    assert_eq!(out, "false", "default must not be below 600000");
}

/// A flipped character in the digest must not verify.
#[test]
fn a_tampered_hash_does_not_verify() {
    let out = run(r#"fn main() {
    let stored = password_hash("pw").unwrap()
    // Rebuild the encoding with a digest that cannot be the real one. Replacing
    // some chosen character would be unreliable: the salt and digest are random,
    // so a run where that character is absent would tamper with nothing and the
    // test would pass while proving nothing.
    let fields = stored.split("$")
    let forged = "$AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA="
    let broken = "$" + fields[1] + "$" + fields[2] + "$" + fields[3] + forged
    let checked = password_verify("pw", broken)
    if checked.is_err() {
        println("rejected")
    } else {
        println(str(checked.unwrap()))
    }
}"#);
    assert!(
        out == "false" || out == "rejected",
        "a tampered hash must not verify, got: {out}"
    );
}

#[test]
fn needs_rehash_is_true_below_the_threshold() {
    let out = run(r#"fn main() {
    let stored = password_hash("pw").unwrap()
    println(str(password_needs_rehash(stored, 1200000).unwrap()))
}"#);
    assert_eq!(out, "true");
}

#[test]
fn needs_rehash_is_false_at_the_threshold() {
    // A stored hash recorded at exactly the threshold is current, not stale.
    let out = run(r#"fn main() {
    let stored = "$pbkdf2-sha256$i=600000$c2FsdA==$xeR41ZKIyEGqUw22hFxMjZYok6ABzk4RpJY4c6qYE0o="
    println(str(password_needs_rehash(stored, 600000).unwrap()))
    println(str(password_needs_rehash(stored, 599999).unwrap()))
}"#);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "false", "at the threshold is current: {out}");
    assert_eq!(lines[1], "false", "above the threshold is current: {out}");
}

/// Cross-check against the published PBKDF2-HMAC-SHA-256 vector described in the
/// module docs: password "password", salt "salt" (base64 `c2FsdA==`), 4096 rounds.
/// The digest is the same value pinned in src/postgres/hmac_tests.rs, so this
/// proves both implementations agree rather than asserting an invented hash.
#[test]
fn verifies_the_published_pbkdf2_sha256_vector() {
    let out = run(r#"fn main() {
    let stored = "$pbkdf2-sha256$i=4096$c2FsdA==$xeR41ZKIyEGqUw22hFxMjZYok6ABzk4RpJY4c6qYE0o="
    println(str(password_verify("password", stored).unwrap()))
    println(str(password_verify("Password", stored).unwrap()))
}"#);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "true", "published vector must verify: {out}");
    assert_eq!(lines[1], "false", "case change must not verify: {out}");
}

#[test]
fn a_malformed_layout_names_the_problem() {
    let out = run(r#"fn main() {
    let bad = password_verify("pw", "not-a-phc-string")
    println(str(bad.is_err()))
    println(bad.err())
}"#);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "true", "full output: {out}");
    assert!(
        lines[1].contains("malformed encoding"),
        "error should name the defect, got: {}",
        lines[1]
    );
}

#[test]
fn an_unknown_algorithm_names_the_algorithm() {
    let out = run(r#"fn main() {
    let bad = password_verify("pw", "$scrypt$i=1000$c2FsdA==$c2FsdA==")
    println(str(bad.is_err()))
    println(bad.err())
}"#);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "true", "full output: {out}");
    assert!(
        lines[1].contains("scrypt") && lines[1].contains("unknown algorithm"),
        "error should name the rejected algorithm, got: {}",
        lines[1]
    );
}

#[test]
fn a_non_numeric_iteration_count_is_named() {
    let out = run(r#"fn main() {
    let bad = password_verify("pw", "$pbkdf2-sha256$i=lots$c2FsdA==$c2FsdA==")
    println(str(bad.is_err()))
    println(bad.err())
}"#);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "true", "full output: {out}");
    assert!(
        lines[1].contains("non-numeric iteration count"),
        "error should name the bad count, got: {}",
        lines[1]
    );
}

#[test]
fn bad_base64_in_the_salt_is_named() {
    let out = run(r#"fn main() {
    let bad = password_verify("pw", "$pbkdf2-sha256$i=1000$not!base64$c2FsdA==")
    println(str(bad.is_err()))
    println(bad.err())
}"#);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "true", "full output: {out}");
    assert!(
        lines[1].contains("salt") && lines[1].contains("base64"),
        "error should name the field and the cause, got: {}",
        lines[1]
    );
}

/// A missing `i=` prefix is a distinct defect from a non-numeric count.
#[test]
fn a_missing_iteration_field_is_named() {
    let out = run(r#"fn main() {
    let bad = password_verify("pw", "$pbkdf2-sha256$600000$c2FsdA==$c2FsdA==")
    println(str(bad.is_err()))
    println(bad.err())
}"#);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "true", "full output: {out}");
    assert!(
        lines[1].contains("missing iteration field"),
        "error should name the missing field, got: {}",
        lines[1]
    );
}

/// A corrupted record must be distinguishable from a wrong password, so it is an
/// Err rather than a silent false.
#[test]
fn corruption_is_an_error_not_a_silent_false() {
    let out = run(r#"fn main() {
    let bad = password_verify("pw", "$pbkdf2-sha256$i=0$c2FsdA==$c2FsdA==")
    println(str(bad.is_err()))
}"#);
    assert_eq!(out, "true", "zero iterations must be rejected");
}

#[test]
fn wrong_argument_types_name_the_parameter() {
    let out = run(r#"fn main() {
    let bad = password_hash(42)
    println(str(bad.is_err()))
    println(bad.err())
}"#);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "true", "full output: {out}");
    assert!(
        lines[1].contains("password_hash: password") && lines[1].contains("must be str"),
        "error should name the parameter, got: {}",
        lines[1]
    );
}
