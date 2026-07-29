//! Agent RPC event application.

use super::{
    super::model::{Event, Message},
    AgentState,
};

impl AgentState {
    pub fn apply(&mut self, event: Event) {
        match event {
            Event::State {
                messages,
                model,
                workspace,
                session,
                ready,
            } => {
                self.messages = messages;
                self.model = model;
                self.workspace = workspace;
                self.session = session;
                self.status = if ready {
                    "Vault provider ready"
                } else {
                    "provider unavailable"
                }
                .into();
            }
            Event::Reply(text) => {
                self.messages.push(Message {
                    role: "assistant".into(),
                    text,
                });
                self.busy = false;
                self.status = "ready".into();
            }
            Event::Error(text) => {
                self.status = text;
                self.busy = false;
            }
        }
    }
}
