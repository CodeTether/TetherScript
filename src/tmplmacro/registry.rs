//! The namespace registry: which macros are reachable from the template being rendered.
//!
//! A call path resolves in two steps. `ns::name` looks in the [`MacroSet`] registered under
//! `ns`, which an `{% import "file" as ns %}` bound. `self::name`, `_self::name`, and bare
//! `name` look in the *current* template's own set, which is what lets a macro call a
//! sibling defined beside it.
//!
//! The registry is data the engine supplies; this component never loads a file, so
//! filesystem policy and capability grants stay entirely the engine's concern.

use std::collections::BTreeMap;

use crate::tmplmacro::macros::MacroSet;

/// Macros reachable from one template: its own, plus one set per imported namespace.
#[derive(Debug, Clone, Default)]
pub struct Registry {
    /// Macros defined by the template currently being rendered.
    pub own: MacroSet,
    /// Imported namespaces, keyed by the alias the `{% import %}` bound.
    pub namespaces: BTreeMap<String, MacroSet>,
}

impl Registry {
    /// A registry holding only the current template's own macros.
    ///
    /// # Arguments
    ///
    /// * `own` — Macros collected from the template being rendered.
    ///
    /// # Returns
    ///
    /// A registry with no imported namespaces.
    ///
    /// # Examples
    ///
    /// ```
    /// use tetherscript::tmplmacro::macros::collect;
    /// use tetherscript::tmplmacro::registry::Registry;
    ///
    /// let own = collect("{% macro row(cfg) %}x{% endmacro %}").unwrap();
    /// let registry = Registry::new(own);
    /// assert!(registry.namespaces.is_empty());
    /// ```
    pub fn new(own: MacroSet) -> Self {
        Self {
            own,
            namespaces: BTreeMap::new(),
        }
    }

    /// Bind a namespace alias to a set of macros.
    ///
    /// # Arguments
    ///
    /// * `namespace` — Alias from an `{% import %}`.
    /// * `set` — Macros collected from the imported template's source.
    ///
    /// # Returns
    ///
    /// The registry, for chaining.
    ///
    /// # Examples
    ///
    /// ```
    /// use tetherscript::tmplmacro::macros::collect;
    /// use tetherscript::tmplmacro::registry::Registry;
    ///
    /// let hero = collect("{% macro hero(cfg) %}h{% endmacro %}").unwrap();
    /// let registry = Registry::default().with("hero", hero);
    /// assert!(registry.namespaces.contains_key("hero"));
    /// ```
    pub fn with(mut self, namespace: &str, set: MacroSet) -> Self {
        self.namespaces.insert(namespace.to_string(), set);
        self
    }
}
