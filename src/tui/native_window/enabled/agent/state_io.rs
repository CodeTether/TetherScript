//! Agent response polling and prompt submission.

use super::{
    super::{
        client::{output::Output, AgentClient},
        model::{Event, Message},
        protocol,
    },
    AgentState,
};

impl AgentState {
    pub(crate) fn receive(&mut self, client: &AgentClient) {
        for output in client.output.try_iter() {
            match output {
                Output::Line(line) => {
                    if let Some(event) = protocol::decode(&line) {
                        self.apply(event);
                    }
                }
                Output::Error(error) if !error.trim().is_empty() => {
                    self.apply(Event::Error(error));
                }
                Output::Error(_) => {}
            }
        }
    }

    pub(crate) fn submit(&mut self, client: &mut AgentClient) -> Result<(), String> {
        let prompt = self.input.trim().to_string();
        if prompt.is_empty() || self.busy {
            return Ok(());
        }
        self.input.clear();
        self.messages.push(Message {
            role: "user".into(),
            text: prompt.clone(),
        });
        self.busy = true;
        self.status = "agent working".into();
        client.send_prompt(&prompt)
    }
}
