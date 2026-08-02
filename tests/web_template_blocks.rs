//! Block-tag coverage for `template_render`.
//!
//! Split from `web_template.rs` so the substitution tests and the control-flow
//! tests stay separate. Braces are escaped (`\{`) because `{` opens a string
//! interpolation hole in tetherscript source.

use std::process::Command;

/// Run a script and return its trimmed stdout, asserting success.
fn stdout_of(source: &str) -> String {
    static CASE: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let case = CASE.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!("tether_tpl_block_{}_{case}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join("case.tether");
    std::fs::write(&path, source).expect("write source");

    let output = Command::new(env!("CARGO_BIN_EXE_tetherscript"))
        .args(["run"])
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

/// Wrap a template expression in a script with a standard context.
fn render(template: &str) -> String {
    stdout_of(&format!(
        "fn main() {{\n\
         let c = map()\n\
         c.name = \"<Riley>\"\n\
         c.on = true\n\
         c.off = false\n\
         c.items = [\"a\", \"b\"]\n\
         c.empty = []\n\
         println(template_render(\"{template}\", c).unwrap())\n\
         }}\n"
    ))
}

/// Same, but expecting the render to fail.
fn render_err(template: &str) -> String {
    stdout_of(&format!(
        "fn main() {{\n\
         let c = map()\n\
         c.on = true\n\
         c.items = [\"a\"]\n\
         c.name = \"x\"\n\
         println(str(template_render(\"{template}\", c).is_err()))\n\
         }}\n"
    ))
}

#[test]
fn an_if_block_takes_the_true_branch() {
    assert_eq!(render("\\{% if on %\\}yes\\{% endif %\\}"), "yes");
}

#[test]
fn a_false_condition_emits_nothing() {
    assert_eq!(render("[\\{% if off %\\}yes\\{% endif %\\}]"), "[]");
}

#[test]
fn else_supplies_the_alternate_branch() {
    assert_eq!(
        render("\\{% if off %\\}a\\{% else %\\}b\\{% endif %\\}"),
        "b"
    );
}

/// `{% if missing %}` is how a view asks whether an optional value is present, so
/// an absent key is false rather than an error.
#[test]
fn a_missing_key_is_falsey_not_an_error() {
    assert_eq!(
        render("\\{% if absent %\\}a\\{% else %\\}b\\{% endif %\\}"),
        "b"
    );
}

/// Tera/Jinja truthiness: an empty list means "nothing to show".
#[test]
fn an_empty_list_is_falsey() {
    assert_eq!(
        render("\\{% if empty %\\}a\\{% else %\\}b\\{% endif %\\}"),
        "b"
    );
}

#[test]
fn a_for_block_repeats_its_body() {
    assert_eq!(
        render("\\{% for i in items %\\}[\\{\\{ i \\}\\}]\\{% endfor %\\}"),
        "[a][b]"
    );
}

#[test]
fn looping_an_empty_list_emits_nothing() {
    assert_eq!(render("[\\{% for i in empty %\\}x\\{% endfor %\\}]"), "[]");
}

/// Escaping must still apply inside a block, or a loop would become an XSS hole.
#[test]
fn escaping_applies_inside_blocks() {
    assert_eq!(
        render("\\{% for i in items %\\}\\{\\{ name \\}\\}\\{% endfor %\\}"),
        "&lt;Riley&gt;&lt;Riley&gt;"
    );
}

#[test]
fn blocks_nest() {
    assert_eq!(
        render(
            "\\{% if on %\\}\\{% for i in items %\\}\\{\\{ i \\}\\}\\{% endfor %\\}\\{% endif %\\}"
        ),
        "ab"
    );
}

/// An inner block must not close the outer one.
#[test]
fn a_nested_if_does_not_terminate_the_outer_block() {
    assert_eq!(
        render("\\{% if on %\\}A\\{% if off %\\}B\\{% endif %\\}C\\{% endif %\\}"),
        "AC"
    );
}

#[test]
fn an_unbalanced_block_is_an_error() {
    assert_eq!(render_err("\\{% if on %\\}unclosed"), "true");
}

/// Iterating a scalar would silently run once; refusing names the mistake.
#[test]
fn looping_a_non_list_is_an_error() {
    assert_eq!(
        render_err("\\{% for i in name %\\}x\\{% endfor %\\}"),
        "true"
    );
}

#[test]
fn an_unsupported_tag_is_an_error() {
    assert_eq!(render_err("\\{% while on %\\}x\\{% endwhile %\\}"), "true");
}

#[test]
fn a_closing_tag_without_an_opener_is_an_error() {
    assert_eq!(render_err("\\{% endif %\\}"), "true");
}
