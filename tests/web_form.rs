//! Coverage for the URL-encoded form and query-string built-ins.
//!
//! These run real `.tether` programs, because the built-ins are only reachable
//! through the interpreter: the codec is a private submodule, so a unit test
//! could not see it, and the script surface is what the the reference application port
//! actually consumes.

use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Tests run in parallel, so each case needs its own file name.
static CASE: AtomicUsize = AtomicUsize::new(0);

fn run_source(src: &str) -> std::process::Output {
    let case = CASE.fetch_add(1, Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!("tether_form_{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir should be creatable");
    let path = dir.join(format!("form_case_{case}.tether"));
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
        .replace("\r\n", "\n")
        .trim_end()
        .to_string()
}

#[test]
fn url_encode_preserves_only_the_unreserved_set() {
    let out = stdout_of("fn main() { println(url_encode(\"aZ09-._~\")) }\n");
    assert_eq!(out, "aZ09-._~");
}

#[test]
fn url_encode_escapes_reserved_characters_in_uppercase_hex() {
    let out = stdout_of("fn main() { println(url_encode(\"a b/c?d&e=f\")) }\n");
    assert_eq!(out, "a%20b%2Fc%3Fd%26e%3Df");
}

/// Space must survive a full round trip even though `+` also decodes to space.
#[test]
fn url_encode_then_decode_round_trips() {
    let src = "fn main() {\n    \
        let original = \"a b/c?d&e=f+g%\"\n    \
        println(url_decode(url_encode(original))? == original)\n}\n";
    assert_eq!(stdout_of(src), "true");
}

#[test]
fn url_decode_treats_plus_as_space() {
    let out = stdout_of("fn main() { println(url_decode(\"Ada+Lovelace\")?) }\n");
    assert_eq!(out, "Ada Lovelace");
}

/// Decoders must accept either hex case, even though encoders emit uppercase.
#[test]
fn url_decode_accepts_uppercase_and_lowercase_hex() {
    let out = stdout_of("fn main() { println(url_decode(\"%2F%2f%3A%3a\")?) }\n");
    assert_eq!(out, "//::");
}

/// Multi-byte UTF-8 is escaped per byte and must reassemble exactly.
#[test]
fn utf8_multibyte_values_round_trip() {
    let src = "fn main() {\n    \
        let encoded = url_encode(\"café→\")\n    \
        println(encoded)\n    \
        println(url_decode(encoded)?)\n}\n";
    let out = stdout_of(src);
    let mut lines = out.lines();
    assert_eq!(lines.next().unwrap(), "caf%C3%A9%E2%86%92");
    assert_eq!(lines.next().unwrap(), "café→");
}

#[test]
fn form_parse_decodes_names_and_values() {
    let src = "fn main() {\n    \
        let f = form_parse(\"name=Ada+Lovelace&year=1843\")?\n    \
        println(f.name)\n    \
        println(f.year)\n}\n";
    assert_eq!(stdout_of(src), "Ada Lovelace\n1843");
}

/// A field submitted with no `=` is present with an empty value, not absent.
#[test]
fn form_parse_gives_a_bare_name_an_empty_value() {
    let src = "fn main() {\n    \
        let f = form_parse(\"flag&other=1\")?\n    \
        println(\"[\" + f.flag + \"]\")\n    \
        println(f.other)\n}\n";
    assert_eq!(stdout_of(src), "[]\n1");
}

#[test]
fn form_parse_keeps_an_explicitly_empty_value() {
    let src = "fn main() {\n    \
        let f = form_parse(\"a=&b=2\")?\n    \
        println(\"[\" + f.a + \"]\")\n    \
        println(f.b)\n}\n";
    assert_eq!(stdout_of(src), "[]\n2");
}

/// The script-facing shape is a map, so a repeated name keeps the last value.
#[test]
fn form_parse_lets_the_last_repeated_key_win() {
    let src = "fn main() {\n    \
        let f = form_parse(\"k=first&k=second\")?\n    \
        println(f.k)\n}\n";
    assert_eq!(stdout_of(src), "second");
}

/// Empty segments from `a=1&&b=2` or a trailing `&` must not create empty keys.
#[test]
fn form_parse_skips_empty_segments() {
    let src = "fn main() {\n    \
        let f = form_parse(\"a=1&&b=2&\")?\n    \
        println(str(f.len()))\n}\n";
    assert_eq!(stdout_of(src), "2");
}

#[test]
fn form_parse_decodes_percent_escapes_in_the_name() {
    let src = "fn main() {\n    \
        let f = form_parse(\"user%20name=Ada\")?\n    \
        println(f[\"user name\"])\n}\n";
    assert_eq!(stdout_of(src), "Ada");
}

#[test]
fn form_encode_percent_encodes_both_sides() {
    let src = "fn main() {\n    \
        let m = map()\n    \
        m[\"a b\"] = \"c&d\"\n    \
        println(form_encode(m)?)\n}\n";
    assert_eq!(stdout_of(src), "a%20b=c%26d");
}

/// Output must be deterministic, so keys are emitted in sorted order.
#[test]
fn form_encode_is_deterministic_across_keys() {
    let src = "fn main() {\n    \
        let m = map()\n    \
        m.zulu = \"1\"\n    \
        m.alpha = \"2\"\n    \
        m.mike = \"3\"\n    \
        println(form_encode(m)?)\n}\n";
    assert_eq!(stdout_of(src), "alpha=2&mike=3&zulu=1");
}

#[test]
fn form_encode_then_parse_round_trips() {
    let src = "fn main() {\n    \
        let m = map()\n    \
        m[\"name\"] = \"Ada Lovelace\"\n    \
        m[\"note\"] = \"a=b&c\"\n    \
        let back = form_parse(form_encode(m)?)?\n    \
        println(back.name)\n    \
        println(back.note)\n}\n";
    assert_eq!(stdout_of(src), "Ada Lovelace\na=b&c");
}

#[test]
fn form_encode_of_an_empty_map_is_empty() {
    let src = "fn main() { println(\"[\" + form_encode(map())? + \"]\") }\n";
    assert_eq!(stdout_of(src), "[]");
}

#[test]
fn truncated_percent_escape_is_a_named_error() {
    let src = "fn main() {\n    \
        let r = url_decode(\"a%2\")\n    \
        println(r.err())\n}\n";
    let out = stdout_of(src);
    assert!(out.contains("truncated percent escape"), "got: {out}");
    assert!(out.contains("%2"), "error should quote the sequence: {out}");
}

#[test]
fn non_hex_percent_escape_is_a_named_error() {
    let src = "fn main() {\n    \
        let r = url_decode(\"a%zz\")\n    \
        println(r.err())\n}\n";
    let out = stdout_of(src);
    assert!(out.contains("invalid percent escape"), "got: {out}");
    assert!(
        out.contains("%zz"),
        "error should quote the sequence: {out}"
    );
}

/// A malformed escape anywhere in a pair must fail the whole parse.
#[test]
fn form_parse_propagates_a_malformed_escape() {
    let src = "fn main() {\n    \
        let r = form_parse(\"a=%zz\")\n    \
        println(r.is_err())\n}\n";
    assert_eq!(stdout_of(src), "true");
}

#[test]
fn non_string_input_is_rejected_by_name() {
    let output = run_source("fn main() { println(url_encode(42)) }\n");
    assert!(!output.status.success(), "an int must not encode");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("url_encode: input"), "got: {stderr}");
}

#[test]
fn form_encode_rejects_a_non_map_by_name() {
    let output = run_source("fn main() { println(form_encode(\"a=1\")) }\n");
    assert!(!output.status.success(), "a str must not encode as a form");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("must be map"), "got: {stderr}");
}
