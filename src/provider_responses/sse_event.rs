//! Responses SSE event accumulation.

use crate::value::Value;

use super::{output_call, output_fields as fields, trace};

pub(super) struct State {
    pub(super) text: String,
    pub(super) calls: Vec<Value>,
    pub(super) response: Option<Value>,
}

impl State {
    pub(super) fn new() -> Self {
        Self {
            text: String::new(),
            calls: Vec::new(),
            response: None,
        }
    }

    pub(super) fn apply(&mut self, event: &Value) -> Result<(), String> {
        trace::event(event);
        match fields::string(event, "type").as_deref() {
            Some("response.output_text.delta") => self.push_delta(event),
            Some("response.output_item.done") => self.push_item(event)?,
            Some("response.completed") => self.response = fields::field(event, "response"),
            _ => {}
        }
        Ok(())
    }

    fn push_delta(&mut self, event: &Value) {
        if let Some(delta) = fields::string(event, "delta") {
            self.text.push_str(&delta);
        }
    }

    fn push_item(&mut self, event: &Value) -> Result<(), String> {
        let Some(item) = fields::field(event, "item") else {
            return Ok(());
        };
        if fields::string(&item, "type").as_deref() == Some("function_call") {
            self.calls.push(output_call::tool_call(&item)?);
        }
        Ok(())
    }
}
