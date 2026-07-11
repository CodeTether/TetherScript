//! Invocation flow for computer authority methods.

use crate::{json, value::Value};

use super::authority::ComputerAuthority;

impl ComputerAuthority {
    pub(crate) fn invoke_method(&self, method: &str, args: &[Value]) -> Result<Value, String> {
        match method {
            "describe" => Ok(self.describe()),
            "trace" | "export_trace_json" => Ok(Value::List(std::rc::Rc::new(
                std::cell::RefCell::new(self.trace.borrow().clone()),
            ))),
            method if method.starts_with("cursor_") => self.invoke_cursor(method, args),
            _ => self.call_checked(method, args),
        }
    }

    fn call_checked(&self, method: &str, args: &[Value]) -> Result<Value, String> {
        let call = super::mapping::prepare(method, args)?;
        self.dispatch(method, call.scope, call.payload)
    }

    pub(crate) fn call_cursor_action(&self, action: &str, params: Value) -> Result<Value, String> {
        let payload = super::value::with_action(action, &params)?;
        let call = super::mapping::scoped(action, payload)?;
        self.dispatch(action, call.scope, call.payload)
    }

    fn dispatch(&self, action: &str, scope: &str, payload: Value) -> Result<Value, String> {
        self.require_scope(scope)?;
        self.trace.borrow_mut().push(payload.clone());
        let body = json::encode_to_string(&payload)?;
        let response = super::transport::post_json(
            &self.endpoint,
            &body,
            self.timeout,
            self.origin.as_deref(),
        )?;
        if response.trim().is_empty() {
            return Ok(Value::Nil);
        }
        let parsed = json::parse_str(&response)
            .map_err(|e| format!("computer.{}: invalid bridge JSON: {}", action, e))?;
        super::response::normalize(action, parsed)
    }
}
