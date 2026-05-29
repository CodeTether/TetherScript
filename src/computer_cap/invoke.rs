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
            _ => self.call_checked(method, args),
        }
    }

    fn call_checked(&self, method: &str, args: &[Value]) -> Result<Value, String> {
        let call = super::mapping::prepare(method, args)?;
        self.require_scope(call.scope)?;
        self.trace.borrow_mut().push(call.payload.clone());
        let body = json::encode_to_string(&call.payload)?;
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
            .map_err(|e| format!("computer.{}: invalid bridge JSON: {}", method, e))?;
        super::response::normalize(method, parsed)
    }
}
