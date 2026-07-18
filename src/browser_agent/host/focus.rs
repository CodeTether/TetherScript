//! Native focus and blur action envelopes.

use crate::browser_agent::Locator;
use crate::value::Value;

use super::state::HostState;

pub(super) fn invoke(
    state: &mut HostState,
    action: &str,
    payload: &Value,
) -> Result<Value, String> {
    let selector = super::value::string_field(payload, "selector")?;
    let quoted = crate::json::encode_to_string(&super::value::string(selector.clone()))?;
    let method = if action == "focus" { "focus" } else { "blur" };
    let apply = format!(
        "(()=>{{let n=document.querySelector({quoted});if(!n)return false;n.{method}();return true;}})()"
    );
    if !state.page.eval_js(&apply)?.truthy() {
        return Err(format!(
            "browser.{action}: selector `{selector}` cannot {method}"
        ));
    }
    let locator = Locator::css(selector);
    if action == "focus" {
        state.focused = Some(locator);
    } else if state.focused.as_ref() == Some(&locator) {
        state.focused = None;
    }
    Ok(Value::Bool(true))
}
