//! `{% import "file" as namespace %}` recording.
//!
//! An import is how a namespace name is bound. It records an alias, and nothing else: no
//! template is loaded here, because loading is the engine's job and this component must
//! stay pure. The engine resolves the recorded path against whatever source it has —
//! filesystem or caller-supplied map — and asks
//! [`crate::tmplmacro::macros::collect`] for that source's definitions.
//!
//! Both quote styles appear in the reference views (`as sc`, `as booking`), so both are
//! accepted.

use crate::tmplmacro::literal::strip_quotes;
use crate::tmplmacro::params_name::is_identifier;
use crate::tmplmacro::tags::tags_of;

/// One recorded import: the template path and the namespace it is bound to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Import {
    /// Template path as written, quotes stripped.
    pub path: String,
    /// Namespace alias, used as the left half of a `ns::name` call path.
    pub namespace: String,
}

/// Record every `{% import %}` in `source`.
///
/// # Arguments
///
/// * `source` — Raw template text.
///
/// # Returns
///
/// The imports in source order; an empty vector when the template imports nothing.
///
/// # Errors
///
/// Returns an error for a malformed import (missing `as`, unquoted path, or a namespace
/// that is not an identifier), and for two imports binding the same namespace, since a
/// silent last-wins would make one component invisible.
///
/// # Examples
///
/// ```
/// use tetherscript::tmplmacro::imports::collect_imports;
///
/// let src = r#"{% import "components/_hero.html.tera" as hero %}"#;
/// let found = collect_imports(src).unwrap();
/// assert_eq!(found[0].namespace, "hero");
/// assert_eq!(found[0].path, "components/_hero.html.tera");
/// assert!(collect_imports("{% import x as y %}").is_err());
/// ```
pub fn collect_imports(source: &str) -> Result<Vec<Import>, String> {
    let mut found: Vec<Import> = Vec::new();
    for tag in tags_of(source) {
        if tag.keyword() != "import" {
            continue;
        }
        let entry = parse_import(tag.body)?;
        if found.iter().any(|old| old.namespace == entry.namespace) {
            return Err(format!(
                "template: namespace `{}` is imported twice; each alias must be unique",
                entry.namespace
            ));
        }
        found.push(entry);
    }
    Ok(found)
}

/// Parse one `import "path" as ns` tag body.
fn parse_import(body: &str) -> Result<Import, String> {
    let rest = body
        .strip_prefix("import")
        .ok_or_else(|| format!("template: `{body}` is not an `import`"))?
        .trim();
    let (path, namespace) = rest
        .rsplit_once(" as ")
        .ok_or_else(|| format!("template: `{{% {body} %}}` must be `import \"file\" as ns`"))?;
    let path = strip_quotes(path.trim())
        .ok_or_else(|| format!("template: import path in `{body}` must be quoted"))?;
    let namespace = namespace.trim();
    if !is_identifier(namespace) {
        return Err(format!(
            "template: import namespace `{namespace}` in `{body}` is not an identifier"
        ));
    }
    Ok(Import {
        path: path.to_string(),
        namespace: namespace.to_string(),
    })
}
