//! Integer resolution for `range()` arguments.
//!
//! Split from [`super::template_range_args`] so each file stays within the line budget.

use crate::value::Value;

/// Resolve an argument to an integer, trying the context first then a literal.
///
/// In lenient mode, a missing key resolves to 0: Tera's own default renders an empty range rather
/// than failing, and a rating display with no data shows no stars.
pub(super) fn int(text: &str, context: &Value, lenient: bool) -> Result<i64, String> {
    match text.parse::<i64>() {
        Ok(number) => Ok(number),
        Err(_) => match super::template_context::lookup_value(context, text) {
            Ok(Value::Int(number)) => Ok(number),
            Ok(Value::Float(number)) => Ok(number as i64),
            Ok(Value::Nil) if lenient => Ok(0),
            Ok(other) => Err(format!(
                "template: range argument `{text}` is {}, not a number",
                other.type_name()
            )),
            Err(_) if lenient => Ok(0),
            Err(_) => Err(format!(
                "template: range argument `{text}` is not a number or a known key"
            )),
        },
    }
}
