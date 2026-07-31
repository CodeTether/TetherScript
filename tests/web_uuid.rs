//! Integration coverage for the UUID built-ins.
//!
//! Version 4 requires the version nibble set to `4` and the variant bits set to
//! binary `10`. A value missing either is not a valid v4 and PostgreSQL's `uuid`
//! type will reject it, so both are asserted across many generated values rather
//! than spot-checked once.

use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static CASE: AtomicUsize = AtomicUsize::new(0);

/// Run a script through the release-independent binary and return its stdout.
fn run(source: &str) -> String {
    let dir = std::env::temp_dir().join(format!("tether_uuid_{}", std::process::id()));
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
fn uuid_v4_has_canonical_shape_and_hyphen_positions() {
    let out = run(r#"
fn main() {
    let id = uuid_v4()
    let parts = id.split("-")
    println(str(id.len()))
    println(str(parts.len()))
    println(str(parts[0].len()) + "-" + str(parts[1].len()) + "-" + str(parts[2].len()) + "-" + str(parts[3].len()) + "-" + str(parts[4].len()))
}
"#);
    let mut lines = out.lines();
    assert_eq!(lines.next(), Some("36"), "full output: {out}");
    assert_eq!(lines.next(), Some("5"), "full output: {out}");
    assert_eq!(lines.next(), Some("8-4-4-4-12"), "full output: {out}");
}

#[test]
fn uuid_v4_sets_version_and_variant_bits() {
    // Checked in-script across many values: character 14 is the version nibble
    // and character 19 is the variant nibble, which must be one of 8, 9, a, b.
    let out = run(r#"
fn main() {
    let mut checked = 0
    let mut bad_version = 0
    let mut bad_variant = 0
    let variants = ["8", "9", "a", "b"]
    let mut i = 0
    while i < 200 {
        let id = uuid_v4()
        let parts = id.split("-")
        let version = parts[2]
        let variant = parts[3]
        if !version.starts_with("4") { bad_version = bad_version + 1 }
        let mut ok = false
        for v in variants {
            if variant.starts_with(v) { ok = true }
        }
        if !ok { bad_variant = bad_variant + 1 }
        checked = checked + 1
        i = i + 1
    }
    println(str(checked))
    println(str(bad_version))
    println(str(bad_variant))
}
"#);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "200", "should have checked 200 values: {out}");
    assert_eq!(lines[1], "0", "every version nibble must be 4: {out}");
    assert_eq!(lines[2], "0", "every variant must be 10xx: {out}");
}

#[test]
fn two_successive_calls_differ() {
    let out = run(r#"
fn main() {
    let a = uuid_v4()
    let b = uuid_v4()
    if a == b { println("same") } else { println("different") }
}
"#);
    assert_eq!(out, "different");
}

#[test]
fn uuid_parse_accepts_a_known_good_literal() {
    let out = run(r#"
fn main() {
    let parsed = uuid_parse("f47ac10b-58cc-4372-a567-0e02b2c3d479")
    println(str(parsed.is_ok()))
    println(parsed.unwrap())
}
"#);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "true", "full output: {out}");
    assert_eq!(lines[1], "f47ac10b-58cc-4372-a567-0e02b2c3d479");
}

#[test]
fn uuid_parse_normalizes_uppercase_to_lowercase() {
    let out = run(r#"
fn main() {
    println(uuid_parse("F47AC10B-58CC-4372-A567-0E02B2C3D479").unwrap())
}
"#);
    assert_eq!(out, "f47ac10b-58cc-4372-a567-0e02b2c3d479");
}

#[test]
fn uuid_parse_rejects_a_too_short_value() {
    let out = run(r#"
fn main() {
    let bad = uuid_parse("f47ac10b-58cc-4372-a567")
    println(str(bad.is_err()))
    println(bad.err())
}
"#);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "true", "full output: {out}");
    assert!(
        lines[1].contains("36 characters"),
        "error should name the length problem, got: {}",
        lines[1]
    );
}

#[test]
fn uuid_parse_rejects_a_bad_character() {
    let out = run(r#"
fn main() {
    let bad = uuid_parse("z47ac10b-58cc-4372-a567-0e02b2c3d479")
    println(str(bad.is_err()))
    println(bad.err())
}
"#);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "true", "full output: {out}");
    assert!(
        lines[1].contains('z') && lines[1].contains("position 0"),
        "error should name the character and position, got: {}",
        lines[1]
    );
}

#[test]
fn uuid_parse_rejects_a_misplaced_hyphen() {
    // Exactly 36 characters, so the length check passes and the hyphen-position
    // check is what must reject it. The first group is 7 chars and the second is
    // 5, moving the first hyphen one position early.
    let out = run(r#"
fn main() {
    let bad = uuid_parse("f47ac10-b58cc-4372-a567-00e02b2c3d47")
    println(str(bad.is_err()))
    println(bad.err())
}
"#);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "true", "full output: {out}");
    assert!(
        // The misplaced hyphen lands where a hex digit belongs, at index 7, so
        // that is the position the error must name.
        lines[1].contains("position 7"),
        "error should name the hyphen position, got: {}",
        lines[1]
    );
}

#[test]
fn uuid_is_valid_agrees_with_parse() {
    let out = run(r#"
fn main() {
    println(str(uuid_is_valid(uuid_v4())))
    println(str(uuid_is_valid("f47ac10b-58cc-4372-a567-0e02b2c3d479")))
    println(str(uuid_is_valid("nope")))
    println(str(uuid_is_valid(42)))
}
"#);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "true", "generated UUIDs must validate: {out}");
    assert_eq!(lines[1], "true");
    assert_eq!(lines[2], "false");
    assert_eq!(lines[3], "false", "a non-str must be false, not an error");
}
