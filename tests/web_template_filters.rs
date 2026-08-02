//! Filter-pipeline coverage for the template engine.
//!
//! Filters are what the reference views need most: 224 `default`, 48 `safe`, and 21
//! `json` uses. `{{ x | json | safe }}` is the idiom for embedding data in a
//! `<script>` block, so it is exercised directly.

use std::process::Command;

/// Run a script and return its trimmed stdout, asserting success.
fn stdout_of(source: &str) -> String {
    static CASE: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let case = CASE.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!("tether_tpl_filt_{}_{case}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join("case.tether");
    std::fs::write(&path, source).expect("write source");

    let output = Command::new(env!("CARGO_BIN_EXE_tetherscript"))
        .arg("run")
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

/// Render a hole body against a fixed context.
fn render(body: &str) -> String {
    let script = String::from("fn main() {\n")
        + "    let c = map()\n"
        + "    c.name = \"<Riley>\"\n"
        + "    c.count = 3\n"
        + "    c.tags = [\"a\", \"b\"]\n"
        + "    c.blank = nil\n"
        + "    println(template_render(\"\\{\\{ "
        + body
        + " \\}\\}\", c).unwrap())\n}\n";
    stdout_of(&script)
}

/// Same, but reading the error instead.
fn render_err(body: &str) -> String {
    let script = String::from("fn main() {\n")
        + "    let c = map()\n"
        + "    c.count = 3\n"
        + "    println(template_render(\"\\{\\{ "
        + body
        + " \\}\\}\", c).err())\n}\n";
    stdout_of(&script)
}

/// The exact idiom the reference uses to embed data in a `<script>` block.
#[test]
fn json_then_safe_emits_unescaped_json() {
    assert_eq!(render("name | json | safe"), "\"<Riley>\"");
    assert_eq!(render("tags | json | safe"), "[\"a\",\"b\"]");
}

/// `safe` is the only filter that changes emission rather than the value.
#[test]
fn safe_suppresses_escaping() {
    assert_eq!(render("name | safe"), "<Riley>");
}

/// Escaping remains the default, so omitting `safe` must still escape.
#[test]
fn without_safe_the_value_is_still_escaped() {
    assert_eq!(render("name"), "&lt;Riley&gt;");
}

#[test]
fn default_supplies_a_missing_key() {
    assert_eq!(render("absent | default(value=1)"), "1");
}

/// `default` must not fire when the value is present, or it would mask real data.
#[test]
fn default_does_not_override_a_present_value() {
    assert_eq!(render("count | default(value=9)"), "3");
}

/// Tera treats null as absent for `default`, and a ported view relies on that.
#[test]
fn nil_counts_as_absent_for_default() {
    assert_eq!(render("blank | default(value=\\\"spare\\\")"), "spare");
}

#[test]
fn a_quoted_default_stays_a_string() {
    assert_eq!(render("absent | default(value=\\\"7\\\")"), "7");
}

#[test]
fn length_counts_a_list() {
    assert_eq!(render("tags | length"), "2");
}

#[test]
fn filters_chain_left_to_right() {
    assert_eq!(render("name | upper | safe"), "<RILEY>");
}

/// Silently ignoring an unknown filter would emit a bare value where a page expects
/// JSON, breaking the consuming script rather than the render.
#[test]
fn an_unknown_filter_is_an_error() {
    let out = render_err("count | nosuchfilter");
    assert!(out.contains("nosuchfilter"), "got: {out}");
}

/// A malformed `default()` must fail even when the key is present, or the mistake
/// would only surface on the rows where the value happens to be missing.
#[test]
fn a_default_without_an_argument_is_an_error_even_when_present() {
    let out = render_err("count | default()");
    assert!(out.contains("needs an argument"), "got: {out}");
}

#[test]
fn an_empty_filter_name_is_an_error() {
    let out = render_err("count | ");
    assert!(out.contains("empty filter name"), "got: {out}");
}

/// A missing key with no `default` must still fail: a typo would otherwise blank a
/// page silently.
#[test]
fn a_missing_key_without_default_is_still_an_error() {
    let out = render_err("absent");
    assert!(out.contains("absent"), "got: {out}");
}
