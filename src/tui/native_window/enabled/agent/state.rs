//! Mutable native agent conversation state.

#[path = "state_apply.rs"]
mod apply;
#[path = "state_io.rs"]
mod io;

use super::model::Message;

pub(super) struct AgentState {
    pub messages: Vec<Message>,
    pub input: String,
    pub status: String,
    pub model: String,
    pub workspace: String,
    pub busy: bool,
    pub mode: String,
}

impl AgentState {
    pub(super) fn new(mode: String) -> Self {
        Self {
            messages: Vec::new(),
            input: String::new(),
            status: "starting agent RPC".into(),
            model: "loading".into(),
            workspace: String::new(),
            busy: false,
            mode,
        }
    }
}
