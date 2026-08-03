//! `{% macro %}` coverage.
//!
//! 85 call sites across the reference application's 159 views, so `macro` gated the last
//! structural construct. Braces are escaped (`\{`) because `{` opens a string
//! interpolation hole in tetherscript source.

use std::process::Command;

/// Run a script and return its trimmed stdout, asserting success.
fn stdout_of(source: &str) -> String {
    static CASE: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let case = CASE.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!("tether_tpl_mac_{}_{case}", std::process::id()));
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

/// One macro definition, as tetherscript source text for a template value.
///
/// Each definition is one physical line, so no tetherscript expression is continued
/// across lines.
fn definitions() -> String {
    String::new()
        + "\\{% macro spacer() %\\}[gap]\\{% endmacro %\\}"
        + "\\{% macro one(x) %\\}<i>\\{\\{ x \\}\\}</i>\\{% endmacro one %\\}"
        + "\\{% macro three(a, b, c) %\\}\\{\\{ a \\}\\}-\\{\\{ b \\}\\}-\\{\\{ c \\}\\}\\{% endmacro %\\}"
        + "\\{% macro badge(kind, size=\\\"sm\\\") %\\}\\{\\{ kind \\}\\}/\\{\\{ size \\}\\}\\{% endmacro badge %\\}"
        + "\\{% macro shout(text) %\\}!\\{\\{ text \\}\\}!\\{% endmacro %\\}"
        + "\\{% macro trust(html) %\\}\\{\\{ html | safe \\}\\}\\{% endmacro %\\}"
        + "\\{% macro flag(on, items) %\\}\\{% if on %\\}Y\\{% else %\\}N\\{% endif %\\}"
        + "\\{% for i in items %\\}[\\{\\{ i \\}\\}]\\{% endfor %\\}\\{% endmacro flag %\\}"
}

/// The macro library every namespaced case calls into.
///
/// Stored under the key `ui`, which *is* the namespace: templates come from a
/// caller-supplied map, so `ui::badge` means "the macro `badge` in the template at key
/// `ui`". That is why no `{% import %}` appears anywhere below.
fn library() -> String {
    String::new()
        + "    let t = map()\n"
        + "    t[\"ui\"] = \""
        + &definitions()
        + "\"\n"
        + "    t[\"recur\"] = \"\\{% macro deep(n) %\\}x\\{\\{ deep(n=n) \\}\\}\\{% endmacro %\\}\"\n"
}

/// Render `template` against the macro library.
///
/// `accessor` reads the outcome: `.unwrap()` for success, `.err()` for a failure.
fn render(template: &str, accessor: &str) -> String {
    let script = String::from("fn main() {\n")
        + "    let c = map()\n"
        + "    c.name = \"<Riley>\"\n"
        + "    c.raw = \"<b>bold</b>\"\n"
        + "    c.on = true\n"
        + "    c.items = [\"a\", \"b\"]\n"
        + &library()
        + "    println(template_render_inherited(\""
        + template
        + "\", c, t)"
        + accessor
        + ")\n}\n";
    stdout_of(&script)
}

/// Render `template` with no template map, so only its own macros are in scope.
fn render_local(template: &str, accessor: &str) -> String {
    let script = String::from("fn main() {\n")
        + "    let c = map()\n"
        + "    c.name = \"<Riley>\"\n"
        + "    c.items = [\"a\", \"b\"]\n"
        + "    println(template_render(\""
        + template
        + "\", c)"
        + accessor
        + ")\n}\n";
    stdout_of(&script)
}

#[test]
fn a_macro_with_no_parameters_renders_its_body() {
    assert_eq!(render("\\{\\{ ui::spacer() \\}\\}", ".unwrap()"), "[gap]");
}

#[test]
fn a_macro_with_one_parameter_binds_it() {
    assert_eq!(
        render("\\{\\{ ui::one(x=\\\"hi\\\") \\}\\}", ".unwrap()"),
        "<i>hi</i>"
    );
}

#[test]
fn a_macro_with_several_parameters_binds_them_by_keyword() {
    assert_eq!(
        render(
            "\\{\\{ ui::three(a=1, b=\\\"two\\\", c=true) \\}\\}",
            ".unwrap()"
        ),
        "1-two-true"
    );
}

/// Keyword order must not matter, or a reordered call site would silently swap values.
#[test]
fn keyword_order_does_not_matter() {
    assert_eq!(
        render("\\{\\{ ui::three(c=3, a=1, b=2) \\}\\}", ".unwrap()"),
        "1-2-3"
    );
}

#[test]
fn a_default_value_is_used_when_the_argument_is_omitted() {
    assert_eq!(
        render("\\{\\{ ui::badge(kind=\\\"new\\\") \\}\\}", ".unwrap()"),
        "new/sm"
    );
}

#[test]
fn a_default_value_can_be_overridden() {
    assert_eq!(
        render(
            "\\{\\{ ui::badge(kind=\\\"new\\\", size=\\\"lg\\\") \\}\\}",
            ".unwrap()"
        ),
        "new/lg"
    );
}

/// The reference's actual shape: one row macro called once per record.
#[test]
fn a_macro_call_inside_a_loop_sees_the_loop_variable() {
    let looped = "\\{% for i in items %\\}\\{\\{ ui::one(x=i) \\}\\}\\{% endfor %\\}";
    assert_eq!(render(looped, ".unwrap()"), "<i>a</i><i>b</i>");
}

/// A body may contain the same constructs a template may, block-structured ones included.
#[test]
fn a_macro_body_may_contain_an_if_and_a_for() {
    assert_eq!(
        render("\\{\\{ ui::flag(on=on, items=items) \\}\\}", ".unwrap()"),
        "Y[a][b]"
    );
}

/// Escaping must survive into a macro body; a naive implementation opens an XSS hole here.
#[test]
fn escaping_applies_inside_a_macro_body() {
    assert_eq!(
        render("\\{\\{ ui::one(x=name) \\}\\}", ".unwrap()"),
        "<i>&lt;Riley&gt;</i>"
    );
}

/// `| safe` inside a body must still opt that one hole out, which is how the reference
/// passes pre-rendered child HTML through a wrapper macro.
#[test]
fn safe_is_honoured_inside_a_macro_body() {
    assert_eq!(
        render("\\{\\{ ui::trust(html=raw) \\}\\}", ".unwrap()"),
        "<b>bold</b>"
    );
}

/// A macro's scope holds only its parameters. Reading a caller variable works by luck at
/// the first call site and breaks at the second, so it is an error instead.
#[test]
fn a_macro_body_cannot_read_a_caller_variable() {
    assert_eq!(
        render("\\{\\{ ui::shout(text=\\\"a\\\") \\}\\}", ".unwrap()"),
        "!a!"
    );
    let leak = "\\{% macro leak(x) %\\}\\{\\{ name \\}\\}\\{% endmacro %\\}\\{\\{ leak(x=1) \\}\\}";
    let err = render_local(leak, ".err()");
    assert!(err.contains("name"), "got: {err}");
}

#[test]
fn a_missing_required_argument_is_named_in_the_error() {
    let out = render("\\{\\{ ui::one() \\}\\}", ".err()");
    assert!(out.contains("needs argument"), "got: {out}");
    assert!(out.contains('x'), "got: {out}");
}

/// A typo'd keyword that is silently ignored renders a subtly wrong page, so it is refused.
#[test]
fn an_unknown_keyword_argument_is_rejected() {
    let out = render("\\{\\{ ui::one(y=1) \\}\\}", ".err()");
    assert!(out.contains("no parameter"), "got: {out}");
    assert!(out.contains('y'), "got: {out}");
}

/// Positional arguments are not supported; the message says so rather than guessing.
#[test]
fn a_positional_argument_is_rejected() {
    let out = render("\\{\\{ ui::one(\\\"hi\\\") \\}\\}", ".err()");
    assert!(out.contains("keyword-only"), "got: {out}");
}

/// Self-recursion must be reported, not overflow the stack and abort the process.
#[test]
fn recursion_is_bounded_and_reported() {
    let out = render("\\{\\{ recur::deep(n=1) \\}\\}", ".err()");
    assert!(out.contains("nested deeper"), "got: {out}");
}

/// `{% endmacro name %}` is an annotation, so the bare form must work too. `badge` and
/// `one` close with a name; `spacer` and `three` close bare, and all of them render.
#[test]
fn both_endmacro_forms_are_accepted() {
    assert_eq!(
        render(
            "\\{\\{ ui::spacer() \\}\\}\\{\\{ ui::badge(kind=\\\"k\\\") \\}\\}",
            ".unwrap()"
        ),
        "[gap]k/sm"
    );
}

/// A definition is a declaration, not output.
#[test]
fn a_macro_defined_but_never_called_emits_nothing() {
    assert_eq!(
        render_local("[\\{% macro un(x) %\\}Z\\{% endmacro %\\}]", ".unwrap()"),
        "[]"
    );
}

/// A macro defined in the template being rendered is callable without a namespace.
#[test]
fn a_local_macro_is_callable_by_bare_name() {
    let local =
        "\\{% macro hi(x) %\\}<b>\\{\\{ x \\}\\}</b>\\{% endmacro hi %\\}\\{\\{ hi(x=name) \\}\\}";
    assert_eq!(render_local(local, ".unwrap()"), "<b>&lt;Riley&gt;</b>");
}

/// `self::` is Tera's spelling for the current template; a ported view keeps working.
#[test]
fn a_self_qualified_call_resolves_locally() {
    let local = "\\{% macro hi(x) %\\}[\\{\\{ x \\}\\}]\\{% endmacro %\\}\
                 \\{\\{ self::hi(x=\\\"q\\\") \\}\\}";
    assert_eq!(render_local(local, ".unwrap()"), "[q]");
}

/// A definition inside an `if` must not confuse block-end finding: the `if` has to run to
/// its own `endif` rather than stopping at the `endmacro`.
#[test]
fn a_macro_inside_an_if_does_not_terminate_the_outer_block() {
    let nested = String::new()
        + "\\{% if on %\\}\\{% macro hi(x) %\\}(\\{\\{ x \\}\\})\\{% endmacro %\\}A\\{% endif %\\}"
        + "\\{\\{ hi(x=1) \\}\\}";
    assert_eq!(render(&nested, ".unwrap()"), "A(1)");
}

/// A definition inside a `for` is the same hazard from the other direction.
#[test]
fn a_macro_inside_a_for_does_not_terminate_the_outer_block() {
    let nested = String::new()
        + "\\{% for i in items %\\}\\{% macro hi(x) %\\}z\\{% endmacro %\\}"
        + "\\{\\{ i \\}\\}\\{% endfor %\\}";
    assert_eq!(render(&nested, ".unwrap()"), "ab");
}

/// An unknown namespace is a missing template, and must name itself.
#[test]
fn an_unknown_namespace_is_an_error() {
    let out = render("\\{\\{ gone::one(x=1) \\}\\}", ".err()");
    assert!(out.contains("gone"), "got: {out}");
}

/// An unknown macro in a known namespace names both.
#[test]
fn an_unknown_macro_is_an_error() {
    let out = render("\\{\\{ ui::nope(x=1) \\}\\}", ".err()");
    assert!(out.contains("nope"), "got: {out}");
}

/// A `{% macro %}` header with no parameter list is a typo, not a shorthand, and is
/// reported even for a macro that is never called.
#[test]
fn a_macro_header_without_a_parameter_list_is_an_error() {
    let out = render_local("\\{% macro hi %\\}z\\{% endmacro %\\}", ".err()");
    assert!(out.contains("parameter list"), "got: {out}");
}

/// An unclosed definition must be reported rather than swallowing the rest of the page.
#[test]
fn an_unbalanced_endmacro_is_an_error() {
    let out = render_local("\\{% macro hi(x) %\\}z", ".err()");
    assert!(out.contains("unbalanced"), "got: {out}");
}
