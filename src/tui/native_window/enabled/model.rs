//! Owned dashboard state detached from runtime values.

use super::super::super::view::View;

pub(super) struct Dashboard {
    pub title: String,
    pub status: String,
    pub tasks: Vec<Task>,
}

pub(super) struct Task {
    pub title: String,
    pub detail: String,
    pub done: bool,
}

impl From<View> for Dashboard {
    fn from(view: View) -> Self {
        let tasks = view.lines.into_iter().map(Task::from).collect();
        Self {
            title: nonempty(view.title, "Tether Tasks"),
            status: nonempty(view.status, "Ready"),
            tasks,
        }
    }
}

impl From<String> for Task {
    fn from(line: String) -> Self {
        let done = line.contains("[x]") || line.contains('✓');
        let line = line
            .trim_start_matches("> ")
            .trim_start_matches("[task] ")
            .trim_start_matches("[x] ")
            .trim_start_matches("[ ] ");
        let (title, detail) = line.split_once(':').unwrap_or((line, ""));
        Self {
            title: title.trim().to_string(),
            detail: detail.trim().to_string(),
            done,
        }
    }
}

fn nonempty(value: String, fallback: &str) -> String {
    if value.trim().is_empty() {
        fallback.into()
    } else {
        value
    }
}
