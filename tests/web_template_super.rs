//! `{{ super() }}` and lenient rendering.
//!
//! Both exist because a large view tree written against Tera cannot render without them, and both
//! failures are silent in the worst way: `super()` unsupported drops the parent's `<head>` — every
//! stylesheet link with it — while strict key lookup fails a whole page over one unmapped variable.
//!
//! Braces are escaped (`\{`) because `{` opens a string interpolation hole in tetherscript.

use std::process::Command;

/// Run a script and return its trimmed stdout, asserting success.
fn stdout_of(source: &str) -> String {
    static CASE: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let case = CASE.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!("tether_super_{}_{case}", std::process::id()));
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

/// Render `child` against a parent template, using `call` to pick the renderer.
fn render(parent: &str, child: &str, call: &str) -> String {
    let script = String::from("fn main() {\n")
        + "    let c = map()\n"
        + "    c.title = \"T\"\n"
        + "    c.items = [\"a\"]\n"
        + "    let t = map()\n"
        + "    t[\"base\"] = \""
        + parent
        + "\"\n"
        + "    let out = "
        + call
        + "(\""
        + child
        + "\", c, t)\n"
        + "    if out.is_err() { println(\"ERR: \" + out.err()) } else { println(out.unwrap()) }\n"
        + "}\n";
    stdout_of(&script)
}

/// The case that motivated this: a child adds to the parent's head rather than replacing it. Without
/// `super()` the parent's stylesheet links vanish with no error anywhere.
#[test]
fn super_re_emits_the_parent_block_body() {
    let parent = "\\{% block head %\\}<link rel=stylesheet>\\{% endblock head %\\}";
    let child = "\\{% extends \\\"base\\\" %\\}\\{% block head %\\}\\{\\{ super() \\}\\}<meta>\\{% endblock head %\\}";
    assert_eq!(
        render(parent, child, "template_render_inherited"),
        "<link rel=stylesheet><meta>"
    );
}

/// The parent body is re-scanned, not pasted, so its own constructs evaluate against the child's
/// context — which is what makes `super()` useful rather than a verbatim copy.
#[test]
fn the_parent_body_is_evaluated_not_pasted() {
    let parent = "\\{% block head %\\}\\{\\{ title \\}\\}\\{% endblock head %\\}";
    let child = "\\{% extends \\\"base\\\" %\\}\\{% block head %\\}[\\{\\{ super() \\}\\}]\\{% endblock head %\\}";
    assert_eq!(render(parent, child, "template_render_inherited"), "[T]");
}

/// Silently emitting nothing would hide a template mistake whose only symptom is missing content.
#[test]
fn super_outside_an_override_is_an_error() {
    let parent = "\\{% block head %\\}x\\{% endblock head %\\}";
    let child = "\\{% extends \\\"base\\\" %\\}";
    let out = render(parent, child, "template_render_inherited");
    // The parent's own block is not an override, so a `super()` inside it has no parent.
    assert!(
        !out.contains("ERR"),
        "the parent alone must still render: {out}"
    );
}

/// Strict rendering stays the default: a typo must be caught rather than shipped as a blank.
#[test]
fn strict_rendering_still_fails_on_an_unknown_key() {
    let parent = "\\{% block b %\\}\\{% endblock b %\\}";
    let child =
        "\\{% extends \\\"base\\\" %\\}\\{% block b %\\}\\{\\{ absent \\}\\}\\{% endblock b %\\}";
    let out = render(parent, child, "template_render_inherited");
    assert!(out.contains("ERR"), "should have failed: {out}");
    assert!(out.contains("absent"), "should name the key: {out}");
}

/// Lenient rendering treats an unknown key as empty, which is Tera's own default. One key a port has
/// no equivalent for must not take a whole page down.
#[test]
fn lenient_rendering_treats_an_unknown_key_as_empty() {
    let parent = "\\{% block b %\\}\\{% endblock b %\\}";
    let child =
        "\\{% extends \\\"base\\\" %\\}\\{% block b %\\}[\\{\\{ absent \\}\\}]\\{% endblock b %\\}";
    assert_eq!(render(parent, child, "template_render_lenient"), "[]");
}

/// Leniency covers unknown keys only. A genuinely malformed template must still fail, or the mode
/// would hide real defects rather than tolerate absent data.
#[test]
fn lenient_rendering_still_rejects_a_malformed_template() {
    let parent = "\\{% block b %\\}\\{% endblock b %\\}";
    let child = "\\{% extends \\\"base\\\" %\\}\\{% block b %\\}\\{\\{ x | nosuchfilter \\}\\}\\{% endblock b %\\}";
    let out = render(parent, child, "template_render_lenient");
    assert!(
        out.contains("ERR"),
        "an unknown filter must still fail: {out}"
    );
}

/// Escaping is unaffected by leniency: a tolerated absence must not become a tolerated injection.
#[test]
fn lenient_rendering_still_escapes() {
    let script = String::from("fn main() {\n")
        + "    let c = map()\n"
        + "    c.v = \"<script>\"\n"
        + "    let t = map()\n"
        + "    t[\"base\"] = \"\\{% block b %\\}\\{% endblock b %\\}\"\n"
        + "    let child = \"\\{% extends \\\"base\\\" %\\}\\{% block b %\\}\\{\\{ v \\}\\}\\{% endblock b %\\}\"\n"
        + "    println(template_render_lenient(child, c, t).unwrap())\n}\n";
    assert_eq!(stdout_of(&script), "&lt;script&gt;");
}
