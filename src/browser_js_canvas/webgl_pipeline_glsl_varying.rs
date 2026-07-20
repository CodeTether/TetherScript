//! Resolution of direct vertex attribute to varying assignments.

pub(crate) fn attribute(
    source: &str,
    varying: &str,
    attributes: &[String],
) -> Option<String> {
    let compact: String = source.chars().filter(|ch| !ch.is_whitespace()).collect();
    let assignment = format!("{varying}=");
    let start = compact.find(&assignment)? + assignment.len();
    let value = compact[start..].split_once(';')?.0;
    attributes
        .iter()
        .find(|attribute| attribute.as_str() == value)
        .cloned()
}
