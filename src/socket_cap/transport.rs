//! Which transport an operation needs.

/// TCP or UDP, granted independently.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Transport {
    Tcp,
    Udp,
}

impl Transport {
    /// CLI flag that grants this transport.
    pub(super) fn flag(self) -> &'static str {
        match self {
            Transport::Tcp => "--grant-tcp",
            Transport::Udp => "--grant-udp",
        }
    }

    /// Human-readable name used in error messages.
    pub(super) fn label(self) -> &'static str {
        match self {
            Transport::Tcp => "TCP",
            Transport::Udp => "UDP",
        }
    }
}
