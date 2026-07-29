//! Fix Runner task, Git evidence, and validation state.

use super::{super::super::state::AgentState, job::Job};
use eframe::egui;

pub(crate) struct State {
    pub(super) task: String,
    pub(super) command: String,
    pub(super) git: Job,
    pub(super) validation: Job,
    active_task: bool,
    message_count: usize,
}

impl State {
    pub fn new(context: &egui::Context) -> Self {
        let mut state = Self {
            task: String::new(),
            command: "cargo test".into(),
            git: Job::idle("Working tree evidence"),
            validation: Job::idle("Validation evidence"),
            active_task: false,
            message_count: 0,
        };
        super::commands::refresh(&mut state.git, context);
        state
    }

    pub fn started(&mut self, message_count: usize) {
        self.active_task = true;
        self.message_count = message_count;
    }

    pub fn poll(&mut self, _context: &egui::Context, agent: &AgentState) -> bool {
        self.git.poll();
        self.validation.poll();
        let finished = self.active_task
            && !agent.busy
            && agent.messages.len() > self.message_count
            && agent
                .messages
                .last()
                .is_some_and(|message| message.role == "assistant");
        if finished {
            self.active_task = false;
        }
        finished
    }
}
