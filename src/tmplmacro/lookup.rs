//! Looking a call up in the registry, with a clear message when it misses.
//!
//! Kept apart from [`crate::tmplmacro::expand`] so resolution policy — and specifically the
//! wording of the two miss cases — is stated once. An unknown namespace and an unknown name
//! within a known namespace are different authoring mistakes and get different messages:
//! the first means a missing or misspelled `{% import %}`, the second a misspelled macro.

use crate::tmplmacro::call::Call;
use crate::tmplmacro::macros::{MacroDef, MacroSet};
use crate::tmplmacro::registry::Registry;

/// Resolve a call to its definition and its cycle key.
///
/// # Arguments
///
/// * `registry` — Macros reachable from the template being rendered.
/// * `call` — The parsed call site.
///
/// # Returns
///
/// The definition, and the `namespace::name` key used for cycle detection by
/// [`crate::tmplmacro::frames::Frames`]. `self` is the key's namespace for a bare call, so
/// two same-named macros in different namespaces are not confused for a cycle.
///
/// # Errors
///
/// Returns an error listing what *is* available when the namespace is unknown, or when the
/// namespace is known but has no macro of that name.
///
/// # Examples
///
/// ```
/// use tetherscript::tmplmacro::call::parse_call;
/// use tetherscript::tmplmacro::lookup::resolve_call;
/// use tetherscript::tmplmacro::macros::collect;
/// use tetherscript::tmplmacro::registry::Registry;
///
/// let registry = Registry::new(collect("{% macro row(c) %}r{% endmacro %}").unwrap());
/// let (def, key) = resolve_call(&registry, &parse_call("row(c=1)").unwrap()).unwrap();
/// assert_eq!(def.body, "r");
/// assert_eq!(key, "self::row");
///
/// let missed = parse_call("nope::row(c=1)").unwrap();
/// assert!(resolve_call(&registry, &missed).is_err());
/// ```
pub fn resolve_call<'r>(
    registry: &'r Registry,
    call: &Call<'_>,
) -> Result<(&'r MacroDef, String), String> {
    let (set, space) = match call.namespace {
        Some(namespace) => (namespace_set(registry, namespace)?, namespace),
        None => (&registry.own, "self"),
    };
    let def = set
        .get(call.name)
        .ok_or_else(|| missing_name(set, space, call.name))?;
    Ok((def, format!("{space}::{}", call.name)))
}

/// Look up a namespace, naming the imported aliases when it is absent.
fn namespace_set<'r>(registry: &'r Registry, namespace: &str) -> Result<&'r MacroSet, String> {
    registry.namespaces.get(namespace).ok_or_else(|| {
        let known: Vec<&str> = registry.namespaces.keys().map(String::as_str).collect();
        format!(
            "template: unknown macro namespace `{namespace}`; \
             add `{{% import \"...\" as {namespace} %}}`. Imported namespaces: [{}]",
            known.join(", ")
        )
    })
}

/// Message for a known namespace that has no macro of this name.
fn missing_name(set: &MacroSet, space: &str, name: &str) -> String {
    let known: Vec<&str> = set.keys().map(String::as_str).collect();
    format!(
        "template: namespace `{space}` defines no macro `{name}`; it defines [{}]",
        known.join(", ")
    )
}
