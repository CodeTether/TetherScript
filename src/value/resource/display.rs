//! Human-readable resource formatting.

use std::fmt;

use super::OwnedResource;

impl fmt::Display for OwnedResource {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "<{} resource ({})>",
            self.kind.type_name(),
            self.lifecycle.label()
        )
    }
}

impl fmt::Debug for OwnedResource {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, formatter)
    }
}
