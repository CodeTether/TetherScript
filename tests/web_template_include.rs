//! `{% include %}` coverage.
//!
//! The reference's base layout pulls in five partials, so `include` gated verbatim reuse
//! of every view. Braces are escaped (`\{`) because `{` opens an interpolation hole.

use std::process::Command;

/// Run a script and return its trimmed stdout, asserting success.
fn stdout_of(source: &str) -> String {
    static CASE: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let case = CASE.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!("tether_tpl_inc_{}_{case}", std::process::id()));
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

/// Render `template` against a fixed set of partials.
///
/// `accessor` reads the outcome: `.unwrap()` for success, `.err()` for a failure.
fn render(template: &str, accessor: &str) -> String {
    let script = String::from("fn main() {\n")
        + "    let c = map()\n"
        + "    c.site = \"<Spotless>\"\n"
        + "    c.items = [\"a\", \"b\"]\n"
        + "    let t = map()\n"
        + "    t[\"nav\"] = \"<nav>\\{\\{ site \\}\\}</nav>\"\n"
        + "    t[\"row\"] = \"[\\{\\{ i \\}\\}]\"\n"
        + "    t[\"selfref\"] = \"\\{% include 'selfref' %\\}\"\n"
        + "    t[\"layout\"] = \"<html>\\{% include 'nav' %\\}"
        + "\\{% block body %\\}\\{% endblock %\\}</html>\"\n"
        + "    println(template_render_inherited(\""
        + template
        + "\", c, t)"
        + accessor
        + ")\n}\n";
    stdout_of(&script)
}

/// An include shares the caller's context, and escaping still applies inside it.
#[test]
fn an_include_renders_the_partial_with_the_current_context() {
    assert_eq!(
        render("\\{% include 'nav' %\\}", ".unwrap()"),
        "<nav>&lt;Spotless&gt;</nav>"
    );
}

/// Double quotes work too; the reference mixes both styles.
#[test]
fn double_quoted_include_names_work() {
    assert_eq!(
        render("\\{% include \\\"nav\\\" %\\}", ".unwrap()"),
        "<nav>&lt;Spotless&gt;</nav>"
    );
}

/// The reference's actual shape: a layout including a partial, extended by a child.
#[test]
fn an_include_inside_an_extended_layout_renders() {
    let child = "\\{% extends 'layout' %\\}\\{% block body %\\}<p>hi</p>\\{% endblock %\\}";
    assert_eq!(
        render(child, ".unwrap()"),
        "<html><nav>&lt;Spotless&gt;</nav><p>hi</p></html>"
    );
}

/// An include inside a loop must see the loop variable, since it shares the scope.
#[test]
fn an_include_inside_a_loop_sees_the_loop_variable() {
    let looped = "\\{% for i in items %\\}\\{% include 'row' %\\}\\{% endfor %\\}";
    assert_eq!(render(looped, ".unwrap()"), "[a][b]");
}

/// `ignore missing` is how the reference makes an optional section optional.
#[test]
fn ignore_missing_renders_nothing_for_an_absent_partial() {
    assert_eq!(
        render("[\\{% include 'gone' ignore missing %\\}]", ".unwrap()"),
        "[]"
    );
}

/// Without the flag, a missing partial must name itself rather than render blank.
#[test]
fn a_missing_partial_without_the_flag_is_an_error() {
    let out = render("\\{% include 'gone' %\\}", ".err()");
    assert!(out.contains("gone"), "got: {out}");
}

/// Self-inclusion must be reported, not overflow the stack and abort the process.
#[test]
fn a_self_including_partial_is_caught() {
    let out = render("\\{% include 'selfref' %\\}", ".err()");
    assert!(out.contains("nested deeper"), "got: {out}");
}
