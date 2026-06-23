//! Structured terminal key event data.

/// A normalized keyboard event parsed from terminal bytes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct KeyEvent {
    pub(super) key: &'static str,
    pub(super) text: Option<String>,
    pub(super) ctrl: bool,
    pub(super) alt: bool,
    pub(super) shift: bool,
}

impl KeyEvent {
    /// Build a non-text key event such as `enter` or `up`.
    pub(super) fn named(key: &'static str) -> Self {
        Self {
            key,
            text: None,
            ctrl: false,
            alt: false,
            shift: false,
        }
    }

    /// Build a printable character key event.
    pub(super) fn text(ch: char) -> Self {
        let mut event = Self::named("char");
        event.text = Some(ch.to_string());
        event
    }

    /// Build a Ctrl-modified character key event.
    pub(super) fn ctrl(ch: char) -> Self {
        let mut event = Self::text(ch);
        event.ctrl = true;
        event
    }

    /// Mark this event as Alt-modified.
    pub(super) fn alt(mut self) -> Self {
        self.alt = true;
        self
    }
}
