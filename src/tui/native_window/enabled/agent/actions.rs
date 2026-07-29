//! Agent response polling and prompt submission actions.

use std::sync::mpsc::TryRecvError;

use super::{
    app::AgentApp,
    client::client_output::Output,
    model::{Event, Message},
    protocol,
};

pub(super) fn poll(app: &mut AgentApp) {
    let Some(client) = &app.client else { return };
    for output in client.output.try_iter() {
        match output {
            Output::Line(line) => {
                if let Some(event) = protocol::decode(&line) {
                    app.state.apply(event);
                }
            }
            Output::Error(error) if !error.trim().is_empty() => {
                app.state.apply(Event::Error(error))
            }
            Output::Error(_) => {}
        }
    }
    if matches!(client.output.try_recv(), Err(TryRecvError::Disconnected)) {
        app.state.apply(Event::Error("agent process exited".into()));
    }
}

pub(super) fn submit(app: &mut AgentApp, prompt: String) {
    app.state.messages.push(Message {
        role: "user".into(),
        text: prompt.clone(),
    });
    app.state.busy = true;
    app.state.status = "agent working".into();
    if let Some(client) = &mut app.client {
        if let Err(error) = client.send_prompt(&prompt) {
            app.state.apply(Event::Error(error));
        }
    }
}
