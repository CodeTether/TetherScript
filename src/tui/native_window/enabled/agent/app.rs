//! Eframe lifecycle for the native agent and CPU visualizer.

#[path = "app_update.rs"]
mod app_update;
#[path = "fix.rs"]
pub(super) mod fix;

use eframe::egui;

use super::{client::AgentClient, disassembly::Snapshot, model::Config, state::AgentState};

pub(super) struct AgentApp {
    pub(super) client: Option<AgentClient>,
    pub(super) state: AgentState,
    pub(super) cpu: Option<Snapshot>,
    pub(super) fix: fix::State,
    pub(super) fix_mode: bool,
}

impl AgentApp {
    pub fn new(config: Config, context: egui::Context) -> Self {
        let mut state = AgentState::new();
        let client = match AgentClient::spawn(&config.agent_path, context.clone()) {
            Ok(client) => Some(client),
            Err(error) => {
                state.status = error;
                None
            }
        };
        let fix_mode = config.mode == "fix";
        let cpu = (!fix_mode).then(|| Snapshot::current().unwrap_or_else(Snapshot::placeholder));
        Self {
            client,
            state,
            cpu,
            fix: fix::State::new(&context),
            fix_mode,
        }
    }
}

impl eframe::App for AgentApp {
    fn update(&mut self, context: &egui::Context, _frame: &mut eframe::Frame) {
        app_update::run(self, context);
    }
}
