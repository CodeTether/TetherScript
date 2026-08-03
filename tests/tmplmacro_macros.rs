//! Integration tests for the `{% macro %}` component in [`tetherscript::tmplmacro`].
//!
//! Covers collection, defaults, both `endmacro` forms, duplicate rejection, call parsing,
//! quote-aware argument parsing, argument/parameter agreement, scope hermeticity, nested
//! expansion, recursion bounds, imports, and lookup failures.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use tetherscript::tmplmacro::args::parse_args;
use tetherscript::tmplmacro::bind::bind;
use tetherscript::tmplmacro::call::{is_call, parse_call};
use tetherscript::tmplmacro::define::skip_definition;
use tetherscript::tmplmacro::expand::expand;
use tetherscript::tmplmacro::frames::{Frames, MAX_DEPTH};
use tetherscript::tmplmacro::imports::collect_imports;
use tetherscript::tmplmacro::macros::{collect, MacroSet};
use tetherscript::tmplmacro::registry::Registry;
use tetherscript::value::Value;

/// Build a `Value::Map` from string-keyed pairs.
fn map_of(pairs: &[(&str, Value)]) -> Value {
    let mut map = HashMap::new();
    for (key, value) in pairs {
        map.insert((*key).to_string(), value.clone());
    }
    Value::Map(Rc::new(RefCell::new(map)))
}

/// Borrow an expansion's child context as a key set.
fn keys_of(context: &Value) -> Vec<String> {
    let Value::Map(map) = context else {
        panic!("expected a map context")
    };
    let mut keys: Vec<String> = map.borrow().keys().cloned().collect();
    keys.sort();
    keys
}

#[test]
fn collects_one_macro_with_params_and_body() {
    let src = r#"pre{% macro badge(kind, size) %}<b>{{ kind }}</b>{% endmacro %}post"#;
    let set = collect(src).unwrap();
    assert_eq!(set.len(), 1);
    let def = &set["badge"];
    assert_eq!(def.name, "badge");
    assert_eq!(def.params[0].name, "kind");
    assert_eq!(def.params[1].name, "size");
    assert_eq!(def.body, "<b>{{ kind }}</b>");
}

#[test]
fn records_defaults_as_raw_literals() {
    let src = r#"{% macro c(a, b="sm", n=3, f=1.5, t=true) %}x{% endmacro %}"#;
    let params = &collect(src).unwrap()["c"].params;
    assert_eq!(params[0].default, None);
    assert_eq!(params[1].default.as_deref(), Some("\"sm\""));
    assert_eq!(params[2].default.as_deref(), Some("3"));
    assert_eq!(params[3].default.as_deref(), Some("1.5"));
    assert_eq!(params[4].default.as_deref(), Some("true"));
}

#[test]
fn accepts_both_endmacro_forms() {
    let named = collect("{% macro a(x) %}A{% endmacro a %}").unwrap();
    let bare = collect("{% macro a(x) %}A{% endmacro %}").unwrap();
    assert_eq!(named["a"].body, "A");
    assert_eq!(bare["a"].body, "A");
}

#[test]
fn collects_two_macros_from_one_source() {
    let src = "{% macro a(x) %}A{% endmacro %}mid{% macro b(y) %}B{% endmacro b %}";
    let set = collect(src).unwrap();
    assert_eq!(set.len(), 2);
    assert_eq!(set["a"].body, "A");
    assert_eq!(set["b"].body, "B");
}

#[test]
fn collects_macro_nested_in_an_if_without_early_close() {
    let src = "{% if flag %}{% macro a(x) %}{% for i in xs %}{{ i }}{% endfor %}\
               {% endmacro %}{% endif %}";
    let set = collect(src).unwrap();
    assert_eq!(set["a"].body, "{% for i in xs %}{{ i }}{% endfor %}");
}

#[test]
fn rejects_a_duplicate_macro_name() {
    let src = "{% macro a(x) %}1{% endmacro %}{% macro a(y) %}2{% endmacro %}";
    let error = collect(src).unwrap_err();
    assert!(error.contains("defined twice"), "{error}");
    assert!(error.contains('a'), "{error}");
}

#[test]
fn rejects_an_unclosed_macro() {
    assert!(collect("{% macro a(x) %}body").unwrap_err().contains("never closed"));
}

#[test]
fn ignores_a_commented_out_endmacro() {
    let set = collect("{% macro a(x) %}A{# {% endmacro %} #}B{% endmacro %}").unwrap();
    assert!(set["a"].body.contains('B'));
}

#[test]
fn definition_site_emits_nothing_and_resumes_after_endmacro() {
    let src = "a{% macro b(x) %}HIDDEN{% endmacro %}z";
    assert_eq!(&src[skip_definition(src, 1).unwrap()..], "z");
}

#[test]
fn parses_namespaced_and_bare_calls() {
    let ns = parse_call(r#"booking::service_calendar(cfg=x)"#).unwrap();
    assert_eq!(ns.namespace, Some("booking"));
    assert_eq!(ns.name, "service_calendar");

    let bare = parse_call("row(cfg=x)").unwrap();
    assert_eq!(bare.namespace, None);
    assert_eq!(bare.name, "row");

    for own in ["self::row(cfg=x)", "_self::row(cfg=x)"] {
        let call = parse_call(own).unwrap();
        assert_eq!((call.namespace, call.name), (None, "row"));
    }
}

#[test]
fn classifies_calls_apart_from_filter_pipelines() {
    assert!(is_call(r#"ui::badge(kind="new")"#));
    assert!(is_call("row()"));
    assert!(!is_call(r#"cfg.html | default(value="")"#));
    assert!(!is_call("cfg.title"));
}

#[test]
fn parses_arguments_containing_quoted_commas_and_parens() {
    let args = parse_args(r#"sep=", ", label="Book (today), fast", n=2"#, "ui::b").unwrap();
    assert_eq!(args.len(), 3);
    assert_eq!(args[0].expression, r#"", ""#);
    assert_eq!(args[1].expression, r#""Book (today), fast""#);
    assert_eq!(args[2].expression, "2");
}

#[test]
fn rejects_a_positional_argument() {
    let error = parse_args(r#""new""#, "ui::badge").unwrap_err();
    assert!(error.contains("keyword-only"), "{error}");
}

#[test]
fn rejects_a_missing_required_argument() {
    let set = collect(r#"{% macro b(kind, size="sm") %}x{% endmacro %}"#).unwrap();
    let args = parse_args("size=\"lg\"", "b").unwrap();
    let error = bind(&set["b"], "b", &args, &Value::Nil).unwrap_err();
    assert!(error.contains("requires argument `kind`"), "{error}");
    assert!(error.contains(r#"b(kind, size="sm")"#), "{error}");
}

#[test]
fn rejects_an_unknown_parameter_name() {
    let set = collect("{% macro b(kind) %}x{% endmacro %}").unwrap();
    let args = parse_args("kinds=1", "b").unwrap();
    let error = bind(&set["b"], "b", &args, &Value::Nil).unwrap_err();
    assert!(error.contains("no parameter `kinds`"), "{error}");
}

#[test]
fn applies_a_default_when_the_argument_is_omitted() {
    let set = collect(r#"{% macro b(kind, size="sm") %}x{% endmacro %}"#).unwrap();
    let args = parse_args(r#"kind="new""#, "b").unwrap();
    let context = bind(&set["b"], "b", &args, &Value::Nil).unwrap();
    let Value::Map(map) = &context else { panic!("map") };
    assert!(matches!(map.borrow()["size"].clone(), Value::Str(s) if *s == "sm"));
}

#[test]
fn child_context_holds_only_parameters_and_no_caller_key() {
    let set = collect(r#"{% macro b(kind, size="sm") %}x{% endmacro %}"#).unwrap();
    let caller = map_of(&[("caller_only", Value::Int(9)), ("kind", Value::Int(1))]);
    let args = parse_args("kind=kind", "b").unwrap();
    let context = bind(&set["b"], "b", &args, &caller).unwrap();
    assert_eq!(keys_of(&context), vec!["kind".to_string(), "size".to_string()]);
}

#[test]
fn resolves_a_dotted_argument_from_the_caller_context() {
    let inner = map_of(&[("title", Value::Int(42))]);
    let caller = map_of(&[("cfg", inner)]);
    let set = collect("{% macro b(t) %}x{% endmacro %}").unwrap();
    let args = parse_args("t=cfg.title", "b").unwrap();
    let context = bind(&set["b"], "b", &args, &caller).unwrap();
    let Value::Map(map) = &context else { panic!("map") };
    assert!(matches!(map.borrow()["t"], Value::Int(42)));
}

#[test]
fn rejects_an_argument_expression_that_does_not_resolve() {
    let set = collect("{% macro b(t) %}x{% endmacro %}").unwrap();
    let args = parse_args("t=nothing", "b").unwrap();
    let error = bind(&set["b"], "b", &args, &map_of(&[])).unwrap_err();
    assert!(error.contains("does not resolve"), "{error}");
}

#[test]
fn expand_returns_body_and_context_without_rendering() {
    let ui = collect(r#"{% macro badge(kind) %}<b>{{ kind }}</b>{% endmacro %}"#).unwrap();
    let registry = Registry::default().with("ui", ui);
    let out = expand(&registry, r#"ui::badge(kind="new")"#, &Value::Nil, &Frames::new()).unwrap();
    // The body comes back as unrendered source: the engine stays the single renderer.
    assert_eq!(out.body, "<b>{{ kind }}</b>");
    assert_eq!(out.frames.depth(), 1);
    assert_eq!(keys_of(&out.context), vec!["kind".to_string()]);
}

#[test]
fn expands_a_nested_macro_call_through_the_returned_frames() {
    let own = collect(
        "{% macro outer(v) %}[{{ self::inner(w=v) }}]{% endmacro %}\
         {% macro inner(w) %}<{{ w }}>{% endmacro %}",
    )
    .unwrap();
    let registry = Registry::new(own);
    let first = expand(&registry, "outer(v=1)", &map_of(&[("v", Value::Int(1))]), &Frames::new())
        .unwrap();
    assert_eq!(first.body, "[{{ self::inner(w=v) }}]");
    // The engine renders `first.body`, meets the inner hole, and passes first.frames down.
    let second = expand(&registry, "self::inner(w=v)", &first.context, &first.frames).unwrap();
    assert_eq!(second.body, "<{{ w }}>");
    assert_eq!(second.frames.depth(), 2);
}

#[test]
fn catches_direct_self_recursion() {
    let own = collect("{% macro a(v) %}{{ self::a(v=v) }}{% endmacro %}").unwrap();
    let registry = Registry::new(own);
    let caller = map_of(&[("v", Value::Int(1))]);
    let first = expand(&registry, "a(v=v)", &caller, &Frames::new()).unwrap();
    let error = expand(&registry, "self::a(v=v)", &first.context, &first.frames).unwrap_err();
    assert!(error.contains("calls itself"), "{error}");
}

#[test]
fn catches_indirect_self_recursion() {
    let own = collect(
        "{% macro a(v) %}{{ self::b(v=v) }}{% endmacro %}\
         {% macro b(v) %}{{ self::a(v=v) }}{% endmacro %}",
    )
    .unwrap();
    let registry = Registry::new(own);
    let caller = map_of(&[("v", Value::Int(1))]);
    let one = expand(&registry, "a(v=v)", &caller, &Frames::new()).unwrap();
    let two = expand(&registry, "self::b(v=v)", &one.context, &one.frames).unwrap();
    let error = expand(&registry, "self::a(v=v)", &two.context, &two.frames).unwrap_err();
    assert!(error.contains("calls itself"), "{error}");
    assert!(error.contains("self::a -> self::b"), "{error}");
}

#[test]
fn bounds_a_chain_of_distinct_macros_at_the_documented_limit() {
    let mut frames = Frames::new();
    for step in 0..MAX_DEPTH {
        frames = frames.push(&format!("ns::m{step}")).unwrap();
    }
    assert_eq!(frames.depth(), MAX_DEPTH);
    let error = frames.push("ns::one_too_many").unwrap_err();
    assert!(error.contains(&format!("limit of {MAX_DEPTH} nested calls")), "{error}");
}

#[test]
fn records_imports_with_both_quote_styles() {
    let src = "{% import \"components/_hero.html.tera\" as hero %}\
               {% import 'components/_cal.html.tera' as booking %}";
    let found = collect_imports(src).unwrap();
    assert_eq!(found.len(), 2);
    assert_eq!(found[0].path, "components/_hero.html.tera");
    assert_eq!(found[0].namespace, "hero");
    assert_eq!(found[1].namespace, "booking");
}

#[test]
fn rejects_malformed_and_duplicate_imports() {
    assert!(collect_imports("{% import x as y %}").unwrap_err().contains("quoted"));
    assert!(collect_imports("{% import \"a\" %}").unwrap_err().contains("must be"));
    let twice = "{% import \"a\" as ns %}{% import \"b\" as ns %}";
    assert!(collect_imports(twice).unwrap_err().contains("imported twice"));
}

#[test]
fn reports_an_unknown_namespace_clearly() {
    let registry = Registry::default().with("ui", MacroSet::new());
    let error = expand(&registry, "nope::x()", &Value::Nil, &Frames::new()).unwrap_err();
    assert!(error.contains("unknown macro namespace `nope`"), "{error}");
    assert!(error.contains("import"), "{error}");
    assert!(error.contains("ui"), "{error}");
}

#[test]
fn reports_an_unknown_macro_name_clearly() {
    let registry = Registry::new(collect("{% macro row(c) %}r{% endmacro %}").unwrap());
    let error = expand(&registry, "rows(c=1)", &Value::Nil, &Frames::new()).unwrap_err();
    assert!(error.contains("defines no macro `rows`"), "{error}");
    assert!(error.contains("[row]"), "{error}");
}

#[test]
fn rejects_a_non_identifier_macro_name_at_collection_time() {
    assert!(collect("{% macro 2col(x) %}y{% endmacro %}").unwrap_err().contains("identifier"));
    assert!(collect("{% macro nope %}y{% endmacro %}").unwrap_err().contains("needs `(...)`"));
}

#[test]
fn accepts_a_parameterless_macro_and_an_empty_call() {
    let registry = Registry::new(collect("{% macro spacer() %}<hr>{% endmacro %}").unwrap());
    let out = expand(&registry, "spacer()", &Value::Nil, &Frames::new()).unwrap();
    assert_eq!(out.body, "<hr>");
    assert!(keys_of(&out.context).is_empty());
}
