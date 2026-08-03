//! Coverage for the date, collection, and text filters.
//!
//! These are the remaining filters the reference views use: 35 `date`, 8 `to_json`, 7
//! `round`, 7 `first`, plus `truncate`. Braces are escaped (`\{`) because `{` opens a
//! string interpolation hole in tetherscript.

use std::process::Command;

/// Run a script and return its trimmed stdout, asserting success.
fn stdout_of(source: &str) -> String {
    static CASE: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let case = CASE.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!("tether_tpl_date_{}_{case}", std::process::id()));
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

/// Render a hole body against a context covering each filterable type.
fn render(body: &str) -> String {
    let script = String::from("fn main() {\n")
        + "    let c = map()\n"
        + "    c.when = 1445412480\n"
        + "    c.items = [\"one\", \"two\", \"three\"]\n"
        + "    c.pi = 3.7\n"
        + "    c.word = \"abcdefghij\"\n"
        + "    c.num = \"42\"\n"
        + "    println(template_render(\"\\{\\{ "
        + body
        + " \\}\\}\", c).unwrap())\n}\n";
    stdout_of(&script)
}

/// Same, reading the error instead.
fn render_err(body: &str) -> String {
    let script = String::from("fn main() {\n")
        + "    let c = map()\n"
        + "    c.word = \"abc\"\n"
        + "    c.when = 0\n"
        + "    println(template_render(\"\\{\\{ "
        + body
        + " \\}\\}\", c).err())\n}\n";
    stdout_of(&script)
}

/// The exact patterns the reference views use. A comma inside the quoted format string is
/// data, not an argument separator — splitting naively truncated the date at "Oct 21".
#[test]
fn date_renders_the_reference_patterns() {
    assert_eq!(
        render("when | date(format=\\\"%b %d, %Y\\\")"),
        "Oct 21, 2015"
    );
    assert_eq!(
        render("when | date(format=\\\"%B %d, %Y\\\")"),
        "October 21, 2015"
    );
    assert_eq!(
        render("when | date(format=\\\"%B %d, %Y at %I:%M %p\\\")"),
        "October 21, 2015 at 07:28 AM"
    );
}

/// `%e` is space-padded day-of-month, which the reference uses.
#[test]
fn date_supports_space_padded_days() {
    assert_eq!(render("when | date(format=\\\"%e\\\")"), "21");
}

/// An unknown specifier survives verbatim so a typo is visible rather than vanishing.
#[test]
fn an_unknown_date_specifier_is_emitted_literally() {
    assert_eq!(render("when | date(format=\\\"%Q\\\")"), "%Q");
}

#[test]
fn date_without_a_format_is_an_error() {
    let out = render_err("when | date()");
    assert!(out.contains("format"), "got: {out}");
}

#[test]
fn to_json_is_an_alias_for_json() {
    assert_eq!(
        render("items | to_json | safe"),
        "[\"one\",\"two\",\"three\"]"
    );
}

#[test]
fn first_and_last_select_list_ends() {
    assert_eq!(render("items | first"), "one");
    assert_eq!(render("items | last"), "three");
}

#[test]
fn round_rounds_a_float_to_an_integer() {
    assert_eq!(render("pi | round"), "4");
}

#[test]
fn truncate_shortens_and_appends_an_ellipsis() {
    assert_eq!(render("word | truncate(length=4)"), "abcd…");
}

/// A custom `end` contains no comma, but the parser must still handle the two-argument
/// form.
#[test]
fn truncate_accepts_a_custom_end() {
    assert_eq!(
        render("word | truncate(length=4, end=\\\"...\\\")"),
        "abcd..."
    );
}

/// A value already short enough must pass through untouched, with no suffix.
#[test]
fn truncate_leaves_short_values_alone() {
    assert_eq!(render("word | truncate(length=99)"), "abcdefghij");
}

#[test]
fn int_and_str_coerce() {
    assert_eq!(render("num | int"), "42");
    assert_eq!(render("pi | str"), "3.7");
}

/// An application-specific filter must be refused with guidance rather than ignored, since
/// silently passing the value through would ship unprocessed content.
#[test]
fn an_application_filter_is_refused_with_guidance() {
    let out = render_err("word | clean_llm_meta");
    assert!(out.contains("clean_llm_meta"), "got: {out}");
    assert!(
        out.contains("context"),
        "should say where it belongs: {out}"
    );
}
