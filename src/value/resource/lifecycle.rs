//! Resource lifecycle state.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Lifecycle {
    Open,
    Closed,
    Cancelled,
}

impl Lifecycle {
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Closed => "closed",
            Self::Cancelled => "cancelled",
        }
    }
}
