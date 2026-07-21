//! Language-visible task state projection.

use super::task::State;

impl State {
    pub(super) fn label(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Scheduled(_, _) => "scheduled",
            Self::Running => "running",
            Self::Complete(_) => "complete",
            Self::Failed(_) => "failed",
            Self::Consumed => "consumed",
        }
    }

    pub(super) fn is_complete(&self) -> bool {
        matches!(self, Self::Complete(_) | Self::Failed(_) | Self::Consumed)
    }
}
