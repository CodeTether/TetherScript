//! Coverage for the validation built-ins.
//!
//! These run real `.tether` programs, because the built-ins are only reachable
//! through the interpreter: the scanners are private submodules, and the script
//! surface is what a real application actually consumes.

use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Tests run in parallel, so each case needs its own file name.
static CASE: AtomicUsize = AtomicUsize::new(0);

fn run_source(src: &str) -> std::process::Output {
    let case = CASE.fetch_add(1, Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!("tether_validate_{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir should be creatable");
    let path = dir.join(format!("validate_case_{case}.tether"));
    std::fs::write(&path, src).expect("source should be writable");
    Command::new(env!("CARGO_BIN_EXE_tetherscript"))
        .arg("run")
        .arg(&path)
        .output()
        .expect("tetherscript should run")
}

/// Run a program and return its trimmed stdout, asserting it succeeded.
fn stdout_of(src: &str) -> String {
    let output = run_source(src);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout)
        .trim_end()
        .to_string()
}

#[test]
fn accepts_ordinary_email_addresses() {
    let out = stdout_of(
        r#"
fn main() {
    println(str(is_email("riley@example.com")))
    println(str(is_email("a.b+tag@sub.example.co.uk")))
    println(str(is_email("x@y.io")))
}
"#,
    );
    assert_eq!(out.lines().collect::<Vec<_>>(), ["true", "true", "true"]);
}

#[test]
fn rejects_malformed_email_addresses() {
    let out = stdout_of(
        r#"
fn main() {
    println(str(is_email("no-at-sign.com")))
    println(str(is_email("two@@example.com")))
    println(str(is_email("@example.com")))
    println(str(is_email("user@localhost")))
    println(str(is_email("a..b@example.com")))
    println(str(is_email("user@exam..ple.com")))
    println(str(is_email("user name@example.com")))
    println(str(is_email("user@example.c")))
    println(str(is_email("")))
}
"#,
    );
    // No consecutive dots, a missing TLD, a one-character TLD, and embedded
    // whitespace must all fail.
    assert!(
        out.lines().all(|line| line == "false"),
        "every case should be false, got: {out}"
    );
}

#[test]
fn accepts_and_rejects_slugs() {
    let out = stdout_of(
        r#"
fn main() {
    println(str(is_slug("my-post")))
    println(str(is_slug("post2")))
    println(str(is_slug("-leading")))
    println(str(is_slug("trailing-")))
    println(str(is_slug("double--hyphen")))
    println(str(is_slug("Upper-Case")))
    println(str(is_slug("has space")))
    println(str(is_slug("")))
}
"#,
    );
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(&lines[..2], ["true", "true"], "full output: {out}");
    assert!(
        lines[2..].iter().all(|line| *line == "false"),
        "leading, trailing, doubled hyphen, case, space, and empty must fail: {out}"
    );
}

#[test]
fn digits_requires_non_empty_ascii() {
    let out = stdout_of(
        r#"
fn main() {
    println(str(is_digits("0123456789")))
    println(str(is_digits("")))
    println(str(is_digits("12a")))
    println(str(is_digits("12 34")))
    println(str(is_digits("١٢٣")))
}
"#,
    );
    // The last case is Arabic-Indic digits: Unicode digits, but not ASCII, so
    // anything parsing them downstream would fail.
    assert_eq!(
        out.lines().collect::<Vec<_>>(),
        ["true", "false", "false", "false", "false"]
    );
}

#[test]
fn normalize_phone_strips_formatting_and_keeps_plus() {
    let out = stdout_of(
        r#"
fn main() {
    println(normalize_phone("(651) 555-0100").unwrap())
    println(normalize_phone("651-555-0100").unwrap())
    println(normalize_phone("+1 651 555 0100").unwrap())
    println(normalize_phone("  651.555.0100  ").unwrap())
}
"#,
    );
    assert_eq!(
        out.lines().collect::<Vec<_>>(),
        ["6515550100", "6515550100", "+16515550100", "6515550100"]
    );
}

#[test]
fn normalize_phone_enforces_the_e164_range() {
    let out = stdout_of(
        r#"
fn main() {
    let short = normalize_phone("12345")
    println(str(short.is_err()))
    println(short.err())
    let long = normalize_phone("1234567890123456")
    println(str(long.is_err()))
    println(long.err())
}
"#,
    );
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "true", "full output: {out}");
    assert!(lines[1].contains("minimum of 7"), "got: {}", lines[1]);
    assert_eq!(lines[2], "true", "full output: {out}");
    assert!(lines[3].contains("maximum of 15"), "got: {}", lines[3]);
}

#[test]
fn normalize_phone_names_an_unexpected_character() {
    let out = stdout_of(
        r#"
fn main() {
    let bad = normalize_phone("651-555-01OO")
    println(str(bad.is_err()))
    println(bad.err())
}
"#,
    );
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "true", "full output: {out}");
    assert!(
        lines[1].contains('O') && lines[1].contains("position"),
        "error should name the character and position, got: {}",
        lines[1]
    );
}

#[test]
fn validate_fields_is_empty_when_everything_passes() {
    let out = stdout_of(
        r#"
fn main() {
    let values = map()
    values.email = "riley@example.com"
    values.slug = "my-post"
    values.zip = "55101"

    let email_rules = map()
    email_rules.required = true
    email_rules.email = true
    let slug_rules = map()
    slug_rules.slug = true
    let zip_rules = map()
    zip_rules.digits = true
    zip_rules.min_len = 5

    let rules = map()
    rules.email = email_rules
    rules.slug = slug_rules
    rules.zip = zip_rules

    let errors = validate_fields(values, rules).unwrap()
    println(str(errors.len()))
}
"#,
    );
    assert_eq!(out, "0", "a fully valid submission must report no errors");
}

#[test]
fn validate_fields_reports_one_message_per_failing_field() {
    let out = stdout_of(
        r#"
fn main() {
    let values = map()
    values.email = "not-an-email"
    values.slug = "Bad--Slug"
    values.zip = "abc"

    let email_rules = map()
    email_rules.email = true
    let slug_rules = map()
    slug_rules.slug = true
    let zip_rules = map()
    zip_rules.digits = true

    let rules = map()
    rules.email = email_rules
    rules.slug = slug_rules
    rules.zip = zip_rules

    let errors = validate_fields(values, rules).unwrap()
    println(str(errors.len()))
    println(errors.email)
    println(errors.slug)
    println(errors.zip)
}
"#,
    );
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "3", "one message per failing field: {out}");
    assert!(lines[1].contains("email"), "got: {}", lines[1]);
    assert!(lines[2].contains("slug"), "got: {}", lines[2]);
    assert!(lines[3].contains("digits"), "got: {}", lines[3]);
}

#[test]
fn required_catches_missing_and_blank_fields() {
    let out = stdout_of(
        r#"
fn main() {
    let values = map()
    values.present = "ok"
    values.blank = "   "

    let required = map()
    required.required = true

    let rules = map()
    rules.present = required
    rules.blank = required
    rules.absent = required

    let errors = validate_fields(values, rules).unwrap()
    println(str(errors.len()))
    println(errors.blank)
    println(errors.absent)
}
"#,
    );
    let lines: Vec<&str> = out.lines().collect();
    // A whitespace-only value is as absent as a missing key for a required field.
    assert_eq!(lines[0], "2", "blank and absent must both fail: {out}");
    assert!(lines[1].contains("required"), "got: {}", lines[1]);
    assert!(lines[2].contains("required"), "got: {}", lines[2]);
}

#[test]
fn optional_rules_skip_an_absent_field() {
    let out = stdout_of(
        r#"
fn main() {
    let values = map()

    let rules = map()
    let optional = map()
    optional.email = true
    optional.min_len = 5
    rules.email = optional

    let errors = validate_fields(values, rules).unwrap()
    println(str(errors.len()))
}
"#,
    );
    assert_eq!(
        out, "0",
        "a field that was never submitted must not fail a non-required rule"
    );
}

#[test]
fn length_rules_count_characters_not_bytes() {
    let out = stdout_of(
        r#"
fn main() {
    let values = map()
    values.name = "café"

    let rules = map()
    let bounds = map()
    bounds.max_len = 4
    rules.name = bounds

    let errors = validate_fields(values, rules).unwrap()
    println(str(errors.len()))
}
"#,
    );
    // "café" is 5 bytes but 4 characters; a byte count would wrongly reject it.
    assert_eq!(out, "0", "max_len must count characters: {out}");
}

#[test]
fn an_unknown_rule_is_an_error_not_a_silent_pass() {
    let out = stdout_of(
        r#"
fn main() {
    let values = map()
    values.email = "riley@example.com"

    let rules = map()
    let bogus = map()
    bogus.nonsense = true
    rules.email = bogus

    let result = validate_fields(values, rules)
    println(str(result.is_err()))
    println(result.err())
}
"#,
    );
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "true", "full output: {out}");
    assert!(
        lines[1].contains("nonsense"),
        "error should name the unknown rule, got: {}",
        lines[1]
    );
}

#[test]
fn non_map_arguments_are_named_errors() {
    let out = stdout_of(
        r#"
fn main() {
    let bad = validate_fields("not a map", map())
    println(str(bad.is_err()))
    println(bad.err())
}
"#,
    );
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "true", "full output: {out}");
    assert!(
        lines[1].contains("values") && lines[1].contains("map"),
        "error should name the parameter and expected type, got: {}",
        lines[1]
    );
}
