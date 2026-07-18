//! Unsupported native host action errors.

pub(super) fn action(action: &str) -> Result<crate::value::Value, String> {
    Err(format!(
        "browser host: native action `{action}` is not implemented"
    ))
}
