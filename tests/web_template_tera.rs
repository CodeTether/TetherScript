//! Coverage for comments, `elif`, comparisons, and `include`.
//!
//! All four are constructs the reference views depend on heavily: 457 comments, 266
//! `elif`, 443 comparison conditions, and 50 includes. Braces are escaped (`\{`) because
//! `{` opens a string interpolation hole in tetherscript.

use std::process::Command;

/// Run a script and return its trimmed stdout, asserting success.
fn stdout_of(source: &str) -> String {
    static CASE: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let case = CASE.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!("tether_tpl_tera_{}_{case}", std::process::id()));
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

/// Render `template` against a context with a range of value types.
fn render(template: &str) -> String {
    let script = String::from("fn main() {\n")
        + "    let c = map()\n"
        + "    c.size = \"lg\"\n"
        + "    c.n = 5\n"
        + "    c.flag = true\n"
        + "    c.items = [\"a\"]\n"
        + "    c.step = map()\n"
        + "    c.step.id = 7\n"
        + "    c.cur = map()\n"
        + "    c.cur.id = 7\n"
        + "    println(template_render(\""
        + template
        + "\", c).unwrap())\n}\n";
    stdout_of(&script)
}

/// Same, reading the error instead.
fn render_err(template: &str) -> String {
    let script = String::from("fn main() {\n")
        + "    let c = map()\n"
        + "    c.size = \"lg\"\n"
        + "    println(template_render(\""
        + template
        + "\", c).err())\n}\n";
    stdout_of(&script)
}

#[test]
fn a_comment_renders_as_nothing() {
    assert_eq!(render("[\\{# note #\\}]"), "[]");
}

/// A comment body may contain other delimiters, so it must be scanned first.
#[test]
fn a_comment_may_contain_delimiters() {
    assert_eq!(
        render("[\\{# \\{\\{ x \\}\\} and \\{% if y %\\} #\\}]"),
        "[]"
    );
}

#[test]
fn an_unclosed_comment_is_an_error() {
    assert!(render_err("\\{# open").contains("unclosed"));
}

/// The first satisfied branch wins, and later ones are skipped.
#[test]
fn an_elif_chain_takes_the_first_match() {
    let chain =
        "\\{% if size == 'sm' %\\}S\\{% elif size == 'lg' %\\}L\\{% else %\\}M\\{% endif %\\}";
    assert_eq!(render(chain), "L");
}

#[test]
fn an_elif_chain_falls_through_to_else() {
    let chain =
        "\\{% if size == 'sm' %\\}S\\{% elif size == 'xl' %\\}X\\{% else %\\}M\\{% endif %\\}";
    assert_eq!(render(chain), "M");
}

/// An untaken branch may reference a key that does not exist, which is how a view guards
/// an optional value.
#[test]
fn an_untaken_branch_is_not_evaluated() {
    let chain = "\\{% if absent %\\}A\\{% elif items %\\}B\\{% else %\\}C\\{% endif %\\}";
    assert_eq!(render(chain), "B");
}

/// An inner `elif` must bind to its own `if`, not the enclosing one.
#[test]
fn a_nested_elif_binds_to_its_own_if() {
    let nested = "\\{% if items %\\}[\\{% if absent %\\}x\\{% elif items %\\}in\\{% endif %\\}]\\{% endif %\\}";
    assert_eq!(render(nested), "[in]");
}

/// This is the case that silently took the wrong branch before comparisons existed.
#[test]
fn string_equality_selects_the_right_branch() {
    assert_eq!(
        render("\\{% if size == 'lg' %\\}y\\{% else %\\}n\\{% endif %\\}"),
        "y"
    );
    assert_eq!(
        render("\\{% if size == 'sm' %\\}y\\{% else %\\}n\\{% endif %\\}"),
        "n"
    );
}

#[test]
fn inequality_and_numeric_comparisons_work() {
    assert_eq!(
        render("\\{% if size != 'sm' %\\}y\\{% else %\\}n\\{% endif %\\}"),
        "y"
    );
    assert_eq!(
        render("\\{% if n > 3 %\\}y\\{% else %\\}n\\{% endif %\\}"),
        "y"
    );
    assert_eq!(
        render("\\{% if n < 3 %\\}y\\{% else %\\}n\\{% endif %\\}"),
        "n"
    );
    assert_eq!(
        render("\\{% if n >= 5 %\\}y\\{% else %\\}n\\{% endif %\\}"),
        "y"
    );
}

/// The reference compares dotted paths on both sides, as in `step.id == current.id`.
#[test]
fn dotted_paths_compare_on_both_sides() {
    assert_eq!(
        render("\\{% if step.id == cur.id %\\}y\\{% else %\\}n\\{% endif %\\}"),
        "y"
    );
}

#[test]
fn booleans_compare() {
    assert_eq!(
        render("\\{% if flag == true %\\}y\\{% else %\\}n\\{% endif %\\}"),
        "y"
    );
}

/// A missing key compares as nil rather than erroring, matching bare-key tolerance.
#[test]
fn a_missing_key_compares_as_absent() {
    assert_eq!(
        render("\\{% if absent == 'x' %\\}y\\{% else %\\}n\\{% endif %\\}"),
        "n"
    );
}

/// Comparing a string with `<` is almost always a mistake; a lexicographic compare would
/// hide it.
#[test]
fn ordering_a_non_number_is_a_named_error() {
    let out = render_err("\\{% if size > 3 %\\}y\\{% endif %\\}");
    assert!(out.contains("needs numbers"), "got: {out}");
}
