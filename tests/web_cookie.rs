//! Coverage for the cookie built-ins.
//!
//! These run real `.tether` programs, because the built-ins are only reachable
//! through the interpreter: the parser and serializer are private submodules, so a
//! unit test could not see them, and the script surface is what a real application
//! session port actually consumes.

use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Tests run in parallel, so each case needs its own file name.
static CASE: AtomicUsize = AtomicUsize::new(0);

fn run_source(src: &str) -> std::process::Output {
    let case = CASE.fetch_add(1, Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!("tether_cookie_{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir should be creatable");
    let path = dir.join(format!("cookie_case_{case}.tether"));
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
fn cookie_parse_reads_multiple_pairs() {
    let src = "fn main() {\n    \
        let jar = cookie_parse(\"id=a1; theme=dark\")\n    \
        println(jar.id)\n    \
        println(jar.theme)\n}\n";
    assert_eq!(stdout_of(src), "a1\ndark");
}

/// Only the first `=` splits a pair, so signed session values survive intact.
#[test]
fn cookie_parse_keeps_equals_signs_inside_the_value() {
    let src = "fn main() {\n    \
        let jar = cookie_parse(\"session=abc=def==\")\n    \
        println(jar.session)\n}\n";
    assert_eq!(stdout_of(src), "abc=def==");
}

#[test]
fn cookie_parse_accepts_an_empty_value() {
    let src = "fn main() {\n    \
        let jar = cookie_parse(\"cleared=; kept=1\")\n    \
        println(\"[\" + jar.cleared + \"]\")\n    \
        println(jar.kept)\n}\n";
    assert_eq!(stdout_of(src), "[]\n1");
}

#[test]
fn cookie_parse_unwraps_a_quoted_value() {
    let src = "fn main() {\n    \
        let jar = cookie_parse(\"id=\\\"quoted value\\\"\")\n    \
        println(jar.id)\n}\n";
    assert_eq!(stdout_of(src), "quoted value");
}

/// Whitespace around `;` separators is not part of the name or value.
#[test]
fn cookie_parse_trims_surrounding_whitespace() {
    let src = "fn main() {\n    \
        let jar = cookie_parse(\"  a = 1 ;  b = 2  \")\n    \
        println(jar.a + \"|\" + jar.b)\n}\n";
    assert_eq!(stdout_of(src), "1|2");
}

/// A browser may send junk this server never set; skip it, do not fail.
#[test]
fn cookie_parse_skips_a_pair_with_no_equals() {
    let src = "fn main() {\n    \
        let jar = cookie_parse(\"broken; ok=1\")\n    \
        println(jar.len())\n    \
        println(jar.ok)\n}\n";
    assert_eq!(stdout_of(src), "1\n1");
}

#[test]
fn cookie_serialize_emits_a_bare_pair_by_default() {
    let src = "fn main() {\n    \
        println(cookie_serialize(\"id\", \"a1\", map())?)\n}\n";
    assert_eq!(stdout_of(src), "id=a1");
}

/// The attribute set and order an Actix session middleware relies on.
#[test]
fn cookie_serialize_emits_every_attribute() {
    let src = "fn main() {\n    \
        let o = map()\n    \
        o.path = \"/\"\n    \
        o.domain = \"example.com\"\n    \
        o.max_age = 604800\n    \
        o.expires = \"Wed, 21 Oct 2015 07:28:00 GMT\"\n    \
        o.same_site = \"Lax\"\n    \
        o.http_only = true\n    \
        o.secure = true\n    \
        println(cookie_serialize(\"id\", \"a1\", o)?)\n}\n";
    assert_eq!(
        stdout_of(src),
        "id=a1; Path=/; Domain=example.com; Max-Age=604800; \
         Expires=Wed, 21 Oct 2015 07:28:00 GMT; SameSite=Lax; HttpOnly; Secure"
    );
}

/// A false flag must be omitted entirely, not emitted as `HttpOnly=false`.
#[test]
fn cookie_serialize_omits_false_flags() {
    let src = "fn main() {\n    \
        let o = map()\n    \
        o.http_only = false\n    \
        o.secure = false\n    \
        println(cookie_serialize(\"id\", \"a1\", o)?)\n}\n";
    assert_eq!(stdout_of(src), "id=a1");
}

#[test]
fn cookie_serialize_accepts_header_style_option_names() {
    let src = "fn main() {\n    \
        let o = map()\n    \
        o[\"Path\"] = \"/\"\n    \
        o[\"HttpOnly\"] = true\n    \
        o[\"SameSite\"] = \"Strict\"\n    \
        println(cookie_serialize(\"id\", \"a1\", o)?)\n}\n";
    assert_eq!(stdout_of(src), "id=a1; Path=/; SameSite=Strict; HttpOnly");
}

#[test]
fn cookie_serialize_normalizes_same_site_casing() {
    let src = "fn main() {\n    \
        let o = map()\n    \
        o.same_site = \"lax\"\n    \
        println(cookie_serialize(\"id\", \"a1\", o)?)\n}\n";
    assert_eq!(stdout_of(src), "id=a1; SameSite=Lax");
}

#[test]
fn cookie_serialize_rejects_an_unknown_same_site_policy() {
    let src = "fn main() {\n    \
        let o = map()\n    \
        o.same_site = \"Sometimes\"\n    \
        let r = cookie_serialize(\"id\", \"a1\", o)\n    \
        println(r.err())\n}\n";
    let out = stdout_of(src);
    assert!(out.contains("Strict, Lax, or None"), "got: {out}");
}

#[test]
fn cookie_serialize_allows_an_empty_value() {
    let src = "fn main() {\n    \
        let o = map()\n    \
        o.max_age = 0\n    \
        println(cookie_serialize(\"id\", \"\", o)?)\n}\n";
    assert_eq!(stdout_of(src), "id=; Max-Age=0");
}

/// What serialize emits, parse must read back.
#[test]
fn serialize_then_parse_round_trips_the_pair() {
    let src = "fn main() {\n    \
        let o = map()\n    \
        o.path = \"/\"\n    \
        o.http_only = true\n    \
        let header = cookie_serialize(\"session\", \"abc=def\", o)?\n    \
        let pair = header.split(\";\")[0]\n    \
        let jar = cookie_parse(pair)\n    \
        println(jar.session)\n}\n";
    assert_eq!(stdout_of(src), "abc=def");
}

// --- Header injection: the security property that matters ---

/// A `;` in the value would end the cookie and let the caller append attributes.
#[test]
fn cookie_serialize_rejects_a_semicolon_in_the_value() {
    let src = "fn main() {\n    \
        let r = cookie_serialize(\"id\", \"a1; Path=/admin\", map())\n    \
        println(r.is_err())\n    \
        println(r.err())\n}\n";
    let out = stdout_of(src);
    assert!(out.starts_with("true"), "must be rejected, got: {out}");
    assert!(out.contains("header injection rejected"), "got: {out}");
}

/// A CRLF would end the header line and allow injecting further headers.
#[test]
fn cookie_serialize_rejects_a_newline_in_the_value() {
    let src = "fn main() {\n    \
        let r = cookie_serialize(\"id\", \"a1\\r\\nSet-Cookie: admin=1\", map())\n    \
        println(r.is_err())\n}\n";
    assert_eq!(stdout_of(src), "true");
}

#[test]
fn cookie_serialize_rejects_a_semicolon_in_the_name() {
    let src = "fn main() {\n    \
        let r = cookie_serialize(\"id; evil=1\", \"a1\", map())\n    \
        println(r.is_err())\n}\n";
    assert_eq!(stdout_of(src), "true");
}

#[test]
fn cookie_serialize_rejects_a_newline_in_the_name() {
    let src = "fn main() {\n    \
        let r = cookie_serialize(\"id\\nSet-Cookie: admin=1\", \"a1\", map())\n    \
        println(r.is_err())\n}\n";
    assert_eq!(stdout_of(src), "true");
}

/// `=` and space in a name would be reparsed as structure.
#[test]
fn cookie_serialize_rejects_a_structural_character_in_the_name() {
    let src = "fn main() {\n    \
        println(cookie_serialize(\"a=b\", \"1\", map()).is_err())\n    \
        println(cookie_serialize(\"a b\", \"1\", map()).is_err())\n    \
        println(cookie_serialize(\"\", \"1\", map()).is_err())\n}\n";
    assert_eq!(stdout_of(src), "true\ntrue\ntrue");
}

/// Injection through an attribute must be refused too, not just name and value.
#[test]
fn cookie_serialize_rejects_injection_through_an_attribute() {
    let src = "fn main() {\n    \
        let o = map()\n    \
        o.path = \"/; HttpOnly\"\n    \
        println(cookie_serialize(\"id\", \"a1\", o).is_err())\n}\n";
    assert_eq!(stdout_of(src), "true");
}

/// A mistyped option is an error: silently ignoring it could drop HttpOnly.
#[test]
fn cookie_serialize_rejects_a_wrongly_typed_option() {
    let src = "fn main() {\n    \
        let o = map()\n    \
        o.http_only = \"yes\"\n    \
        let r = cookie_serialize(\"id\", \"a1\", o)\n    \
        println(r.err())\n}\n";
    let out = stdout_of(src);
    assert!(out.contains("must be bool"), "got: {out}");
}

#[test]
fn cookie_serialize_reports_a_non_map_options_argument() {
    let src = "fn main() {\n    \
        let r = cookie_serialize(\"id\", \"a1\", \"nope\")\n    \
        println(r.err())\n}\n";
    let out = stdout_of(src);
    assert!(out.contains("options must be map"), "got: {out}");
}
