//! Template inheritance coverage for `template_render_inherited`.
//!
//! Inheritance is the construct that blocked every reference view: 200 `extends`
//! and 496 `block` uses across the reference application's 159 Tera templates. Braces are
//! escaped (`\{`) because `{` opens a string interpolation hole in tetherscript.

use std::process::Command;

/// Run a script and return its trimmed stdout, asserting success.
fn stdout_of(source: &str) -> String {
    static CASE: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let case = CASE.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!("tether_tpl_inh_{}_{case}", std::process::id()));
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

/// Render `child` against a two-block parent named `base`.
///
/// Braces are doubled for `format!`, then singled again in the emitted script,
/// where `\{` escapes tetherscript's interpolation hole.
fn render(child: &str) -> String {
    let parent = "<\\{% block head %\\}H\\{% endblock head %\\}\
                  |\\{% block body %\\}B\\{% endblock %\\}>";
    let script = String::from("fn main() {\n")
        + "    let c = map()\n"
        + "    c.title = \"T\"\n"
        + "    c.items = [\"a\", \"b\"]\n"
        + "    c.on = true\n"
        + "    let t = map()\n"
        + "    t[\"base\"] = \""
        + parent
        + "\"\n    println(template_render_inherited(\""
        + child
        + "\", c, t).unwrap())\n}\n";
    stdout_of(&script)
}

#[test]
fn a_child_block_replaces_the_parent_default() {
    assert_eq!(
        render("\\{% extends \\\"base\\\" %\\}\\{% block body %\\}mine\\{% endblock %\\}"),
        "<H|mine>"
    );
}

/// A block the child does not override must keep the parent's content, which is
/// what makes a block body a default rather than a requirement.
#[test]
fn an_unoverridden_block_keeps_its_default() {
    assert_eq!(
        render("\\{% extends \\\"base\\\" %\\}\\{% block head %\\}mine\\{% endblock %\\}"),
        "<mine|B>"
    );
}

#[test]
fn a_child_with_no_blocks_renders_the_parent_unchanged() {
    assert_eq!(render("\\{% extends \\\"base\\\" %\\}"), "<H|B>");
}

/// The reference views use single quotes, which Tera permits.
#[test]
fn single_quoted_template_names_work() {
    assert_eq!(
        render("\\{% extends 'base' %\\}\\{% block body %\\}q\\{% endblock %\\}"),
        "<H|q>"
    );
}

/// `{% endblock name %}` and bare `{% endblock %}` are both valid.
#[test]
fn endblock_may_name_its_block() {
    assert_eq!(
        render("\\{% extends \\\"base\\\" %\\}\\{% block body %\\}x\\{% endblock body %\\}"),
        "<H|x>"
    );
}

#[test]
fn a_block_body_may_contain_control_flow() {
    assert_eq!(
        render(
            "\\{% extends \\\"base\\\" %\\}\\{% block body %\\}\\{% if on %\\}\\{% for i in items %\\}\\{\\{ i \\}\\}\\{% endfor %\\}\\{% endif %\\}\\{% endblock %\\}"
        ),
        "<H|ab>"
    );
}

#[test]
fn interpolation_works_inside_an_overridden_block() {
    assert_eq!(
        render("\\{% extends \\\"base\\\" %\\}\\{% block body %\\}\\{\\{ title \\}\\}\\{% endblock %\\}"),
        "<H|T>"
    );
}

/// Whitespace before `extends` is ordinary formatting, not content.
#[test]
fn leading_whitespace_before_extends_is_allowed() {
    assert_eq!(
        render("\\n  \\{% extends \\\"base\\\" %\\}\\{% block body %\\}w\\{% endblock %\\}"),
        "<H|w>"
    );
}
