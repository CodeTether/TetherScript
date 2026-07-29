//! Shared panel presentation helpers.

use super::model::Dashboard;

pub(super) fn progress_label(dashboard: &Dashboard) -> String {
    let done = dashboard.tasks.iter().filter(|task| task.done).count();
    format!("{done} of {} complete", dashboard.tasks.len())
}
