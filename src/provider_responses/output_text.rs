//! Responses assistant text extraction.

use crate::value::Value;

use super::output_fields as fields;

pub(super) fn parts(item: &Value) -> Vec<String> {
    if let Some(Value::Str(text)) = fields::field(item, "content") {
        return vec![text.to_string()];
    }
    let Some(content) = fields::list(item, "content") else {
        return Vec::new();
    };
    let parts = content.borrow();
    parts.iter().filter_map(part_text).collect()
}

fn part_text(value: &Value) -> Option<String> {
    let kind = fields::string(value, "type");
    if matches!(kind.as_deref(), Some("output_text" | "text") | None) {
        return fields::string(value, "text");
    }
    None
}
