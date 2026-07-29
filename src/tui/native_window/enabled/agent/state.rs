//! Mutable native agent conversation state.

#[path = "state_apply.rs"]
mod apply;

use super::model::Message;

pub(super) struct AgentState {
    pub messages: Vec<Message>,
    pub input: String,
    pub status: String,
    pub model: String,
    pub workspace: String,
    pub session: String,
    pub busy: bool,
}

impl AgentState {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            input: String::new(),
            status: "starting agent RPC".into(),
            model: "loading".into(),
            workspace: String::new(),
            session: String::new(),
            busy: false,
        }
    }
}
