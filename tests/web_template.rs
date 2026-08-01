//! Coverage for the dependency-free template built-ins.
//!
//! These run real `.tether` programs, because the renderer is only reachable
//! through the interpreter. Note that tetherscript reads `{` in a string literal
//! as the start of an interpolation hole, so every template below writes its
//! braces escaped as `\{\{` and `\}\}` — the same convention `tests/web_route.rs`
//! uses for `\{id\}` patterns.

use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Tests run in parallel, so each case needs its own file name.
static CASE: AtomicUsize = AtomicUsize::new(0);

fn run_source(src: &str) -> std::process::Output {
    let case = CASE.fetch_add(1, Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!("tether_template_{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir should be creatable");
    let path = dir.join(format!("template_case_{case}.tether"));
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
fn escapes_each_special_character() {
    let out = stdout_of("fn main() { println(html_escape(\"<a href='x'>&\\\"\")) }\n");
    assert_eq!(out, "&lt;a href=&#39;x&#39;&gt;&amp;&quot;");
}

/// The ampersand must be replaced first, or `<` would render `&amp;lt;`.
#[test]
fn ampersand_is_escaped_exactly_once() {
    let out = stdout_of("fn main() { println(html_escape(\"a & <b>\")) }\n");
    assert_eq!(out, "a &amp; &lt;b&gt;");
    assert!(!out.contains("&amp;lt"), "double-escaped: {out}");
}

#[test]
fn attr_escaping_neutralizes_unquoted_breakouts() {
    let out = stdout_of("fn main() { println(html_attr(\"a b=c/d`e\")) }\n");
    assert_eq!(out, "a&#32;b&#61;c&#47;d&#96;e");
}

#[test]
fn substitutes_a_present_key() {
    let out = stdout_of(
        "fn main() {\n    let c = map()\n    c.title = \"Home\"\n    \
         println(template_render(\"<h1>\\{\\{ title \\}\\}</h1>\", c).unwrap())\n}\n",
    );
    assert_eq!(out, "<h1>Home</h1>");
}

#[test]
fn escapes_interpolated_values_by_default() {
    let out = stdout_of(
        "fn main() {\n    let c = map()\n    c.name = \"<script>\"\n    \
         println(template_render(\"\\{\\{ name \\}\\}\", c).unwrap())\n}\n",
    );
    assert_eq!(out, "&lt;script&gt;", "default must escape");
}

#[test]
fn triple_brace_leaves_markup_intact() {
    let out = stdout_of(
        "fn main() {\n    let c = map()\n    c.body = \"<em>hi</em>\"\n    \
         println(template_render(\"\\{\\{\\{ body \\}\\}\\}\", c).unwrap())\n}\n",
    );
    assert_eq!(out, "<em>hi</em>");
}

#[test]
fn resolves_a_dotted_lookup() {
    let out = stdout_of(
        "fn main() {\n    let user = map()\n    user.name = \"Riley\"\n    \
         let c = map()\n    c.user = user\n    \
         println(template_render(\"hi \\{\\{ user.name \\}\\}\", c).unwrap())\n}\n",
    );
    assert_eq!(out, "hi Riley");
}

#[test]
fn tolerates_whitespace_inside_the_braces() {
    let out = stdout_of(
        "fn main() {\n    let c = map()\n    c.x = \"1\"\n    \
         println(template_render(\"[\\{\\{x\\}\\}][\\{\\{   x   \\}\\}]\", c).unwrap())\n}\n",
    );
    assert_eq!(out, "[1][1]");
}

/// A typo must fail loudly rather than blanking part of the page.
#[test]
fn unknown_key_is_a_named_error() {
    let out = stdout_of(
        "fn main() {\n    let c = map()\n    c.title = \"Home\"\n    \
         let r = template_render(\"\\{\\{ ttile \\}\\}\", c)\n    \
         println(str(r.is_err()))\n    println(r.err())\n}\n",
    );
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "true", "full output: {out}");
    assert!(lines[1].contains("unknown key"), "got: {}", lines[1]);
    assert!(
        lines[1].contains("ttile"),
        "must name the key: {}",
        lines[1]
    );
}

#[test]
fn unclosed_placeholder_is_a_named_error() {
    let out = stdout_of(
        "fn main() {\n    let c = map()\n    \
         let r = template_render(\"<p>\\{\\{ title\", c)\n    \
         println(str(r.is_err()))\n    println(r.err())\n}\n",
    );
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "true", "full output: {out}");
    assert!(lines[1].contains("unclosed"), "got: {}", lines[1]);
}

#[test]
fn raw_render_escapes_nothing() {
    let out = stdout_of(
        "fn main() {\n    let c = map()\n    c.v = \"a & b\"\n    \
         println(template_render_raw(\"\\{\\{ v \\}\\}\", c).unwrap())\n}\n",
    );
    assert_eq!(out, "a & b");
}

#[test]
fn renders_a_small_html_page() {
    let out = stdout_of(
        "fn main() {\n    let c = map()\n    c.title = \"Spotless\"\n    \
         c.heading = \"Bins & Cans\"\n    \
         let tpl = \"<html><head><title>\\{\\{ title \\}\\}</title></head>\"\n    \
         let body = \"<body><h1>\\{\\{ heading \\}\\}</h1></body></html>\"\n    \
         println(template_render(tpl + body, c).unwrap())\n}\n",
    );
    assert_eq!(
        out,
        "<html><head><title>Spotless</title></head><body><h1>Bins &amp; Cans</h1></body></html>"
    );
}
