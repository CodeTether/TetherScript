//! Named import alias parsing for module resources.

pub(crate) fn from_statement(statement: &str) -> Vec<(String, String)> {
    let Some((_, rest)) = statement.split_once('{') else {
        return Vec::new();
    };
    let Some((names, _)) = rest.split_once('}') else {
        return Vec::new();
    };
    names.split(',').filter_map(alias).collect()
}

fn alias(part: &str) -> Option<(String, String)> {
    let value = part.trim();
    if value.is_empty() {
        return None;
    }
    let (imported, local) = value.split_once(" as ").unwrap_or((value, value));
    Some((imported.trim().into(), local.trim().into()))
}
