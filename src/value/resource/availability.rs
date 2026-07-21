//! Closed, cancelled, and expired operation rejection.

use super::{lifecycle::Lifecycle, OwnedResource};

impl OwnedResource {
    pub(super) fn unavailable(&self, operation: &str) -> Option<String> {
        match self.lifecycle {
            Lifecycle::Closed => Some(format!(
                "{}.{operation}: resource is closed",
                self.kind.type_name()
            )),
            Lifecycle::Cancelled => Some(format!(
                "{}.{operation}: resource is cancelled",
                self.kind.type_name()
            )),
            Lifecycle::Open if self.deadline.expired() => Some(format!(
                "{}.{operation}: deadline exceeded",
                self.kind.type_name()
            )),
            Lifecycle::Open => None,
        }
    }
}
