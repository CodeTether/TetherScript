//! Background command state and event-driven result delivery.

use eframe::egui;
use std::sync::mpsc::{self, Receiver};

pub(super) struct Job {
    pub label: String,
    pub output: String,
    pub running: bool,
    result: Option<Receiver<Result<String, String>>>,
}

impl Job {
    pub fn idle(label: &str) -> Self {
        Self {
            label: label.into(),
            output: "Not run".into(),
            running: false,
            result: None,
        }
    }

    pub fn start<F>(&mut self, label: &str, context: &egui::Context, work: F)
    where
        F: FnOnce() -> Result<String, String> + Send + 'static,
    {
        if self.running {
            return;
        }
        self.label = label.into();
        self.output = "Running…".into();
        self.running = true;
        let (sender, receiver) = mpsc::channel();
        self.result = Some(receiver);
        let context = context.clone();
        std::thread::spawn(move || {
            let _ = sender.send(work());
            context.request_repaint();
        });
    }

    pub fn poll(&mut self) {
        let Some(receiver) = &self.result else { return };
        let Ok(result) = receiver.try_recv() else {
            return;
        };
        self.output = result.unwrap_or_else(|error| {
            format!(
                "ERROR
{error}"
            )
        });
        self.running = false;
        self.result = None;
    }
}
