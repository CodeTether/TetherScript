//! Native agent configuration and protocol values.

use crate::value::Value;

pub(super) struct Config {
    pub title: String,
    pub agent_path: String,
    pub mode: String,
}
pub(super) struct Message {
    pub role: String,
    pub text: String,
}
pub(super) enum Event {
    State {
        messages: Vec<Message>,
        model: String,
        workspace: String,
        ready: bool,
    },
    Reply(String),
    Error(String),
}

impl Config {
    pub(super) fn parse(value: &Value) -> Result<Self, String> {
        let Value::Map(map) = value else {
            return Err("tui_native_agent: config must be map".into());
        };
        let map = map.borrow();
        Ok(Self {
            title: field(&map, "title", "tetherscript agent"),
            agent_path: field(&map, "agent_path", "examples/agent_tui.tether"),
            mode: field(&map, "mode", "agent"),
        })
    }
}

pub(super) fn require(value: &Value, name: &str) -> Result<(), String> {
    match value {
        Value::Capability(capability) if capability.kind == name => Ok(()),
        _ => Err(format!("tui_native_agent: missing capability: {name}")),
    }
}

fn field(map: &std::collections::HashMap<String, Value>, key: &str, default: &str) -> String {
    map.get(key)
        .map(Value::to_string)
        .unwrap_or_else(|| default.into())
}
