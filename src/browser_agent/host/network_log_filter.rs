//! Native network-event query filters.

use crate::browser_session::NetworkEvent;
use crate::value::Value;

pub(super) fn failed_only(payload: &Value) -> Result<bool, String> {
    let Value::Map(map) = payload else {
        return Err("browser host: action payload must be map".into());
    };
    match map.borrow().get("failed_only") {
        Some(Value::Bool(value)) => Ok(*value),
        Some(value) => Err(format!(
            "browser.network_log: `failed_only` must be bool, got {}",
            value.type_name()
        )),
        None => Ok(false),
    }
}

pub(super) fn failed(event: &NetworkEvent) -> bool {
    event.status.is_none_or(|status| status >= 400)
        || event.route_result.as_deref() == Some("failed")
}
