//! Persistent in-process browser session state.
//!
//! `BrowserSession` is the deterministic/offline companion to the live browser
//! capability. It keeps the mutable page/session state that agents need between
//! navigation calls while reusing the lightweight HTML/CSS parser in `browser`.

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::browser::{extract_embedded_css, parse_html, Document};

const BLANK_URL: &str = "about:blank";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsoleEvent {
    pub level: String,
    pub message: String,
    pub timestamp_ms: u128,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkEvent {
    pub method: String,
    pub url: String,
    pub status: Option<u16>,
    pub timestamp_ms: u128,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceEvent {
    pub action: String,
    pub detail: String,
    pub timestamp_ms: u128,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScrollState {
    pub x: i64,
    pub y: i64,
}

#[derive(Debug, Clone, PartialEq)]
struct HistoryEntry {
    url: String,
    html: String,
    css: String,
    document: Document,
    focus: Option<String>,
    scroll: ScrollState,
}

/// Persistent browser session model for deterministic/offline navigation.
#[derive(Debug, Clone, PartialEq)]
pub struct BrowserSession {
    pub url: String,
    pub history: Vec<String>,
    pub document: Document,
    pub html: String,
    pub css: String,
    pub console: Vec<ConsoleEvent>,
    pub network: Vec<NetworkEvent>,
    pub storage: HashMap<String, String>,
    pub focus: Option<String>,
    pub scroll: ScrollState,
    pub trace: Vec<TraceEvent>,
    pub offline: bool,
    history_entries: Vec<HistoryEntry>,
    history_index: usize,
}

impl Default for BrowserSession {
    fn default() -> Self {
        Self::new()
    }
}

impl BrowserSession {
    /// Create a blank persistent browser session.
    pub fn new() -> Self {
        let document = parse_html("");
        let entry = HistoryEntry {
            url: BLANK_URL.to_string(),
            html: String::new(),
            css: String::new(),
            document: document.clone(),
            focus: None,
            scroll: ScrollState { x: 0, y: 0 },
        };
        Self {
            url: BLANK_URL.to_string(),
            history: vec![BLANK_URL.to_string()],
            document,
            html: String::new(),
            css: String::new(),
            console: Vec::new(),
            network: Vec::new(),
            storage: HashMap::new(),
            focus: None,
            scroll: ScrollState { x: 0, y: 0 },
            trace: vec![TraceEvent::new("new", BLANK_URL)],
            offline: false,
            history_entries: vec![entry],
            history_index: 0,
        }
    }

    /// Load HTML into the current history entry without changing the current URL.
    pub fn load_html(&mut self, html: impl Into<String>) {
        let html = html.into();
        self.replace_current(self.url.clone(), html, String::new(), "load_html");
    }

    /// Navigate to an in-memory HTML page and append it to session history.
    pub fn goto_html(&mut self, url: impl Into<String>, html: impl Into<String>) {
        let url = url.into();
        let html = html.into();
        self.truncate_forward_history();
        let entry = Self::entry_from_parts(url.clone(), html, String::new());
        self.history_entries.push(entry);
        self.history_index = self.history_entries.len() - 1;
        self.apply_history_entry();
        self.network.push(NetworkEvent::new("GET", &url, Some(200)));
        self.trace.push(TraceEvent::new("goto_html", &url));
    }

    /// Enable or disable offline mode. Offline mode is persistent session state;
    /// callers can inspect it before attempting external navigation.
    pub fn offline(&mut self, offline: bool) {
        self.offline = offline;
        self.trace.push(TraceEvent::new(
            "offline",
            if offline { "true" } else { "false" },
        ));
    }

    /// Reload the current in-memory page from its stored history entry.
    pub fn reload(&mut self) {
        self.apply_history_entry();
        self.trace.push(TraceEvent::new("reload", &self.url));
    }

    /// Move backward in history. Returns true if navigation occurred.
    pub fn back(&mut self) -> bool {
        if self.history_index == 0 {
            self.trace.push(TraceEvent::new("back", "at-start"));
            return false;
        }
        self.persist_current_view_state();
        self.history_index -= 1;
        self.apply_history_entry();
        self.trace.push(TraceEvent::new("back", &self.url));
        true
    }

    /// Move forward in history. Returns true if navigation occurred.
    pub fn forward(&mut self) -> bool {
        if self.history_index + 1 >= self.history_entries.len() {
            self.trace.push(TraceEvent::new("forward", "at-end"));
            return false;
        }
        self.persist_current_view_state();
        self.history_index += 1;
        self.apply_history_entry();
        self.trace.push(TraceEvent::new("forward", &self.url));
        true
    }

    fn replace_current(&mut self, url: String, html: String, css: String, action: &str) {
        self.persist_current_view_state();
        self.history_entries[self.history_index] = Self::entry_from_parts(url.clone(), html, css);
        self.apply_history_entry();
        self.trace.push(TraceEvent::new(action, &url));
    }

    fn entry_from_parts(url: String, html: String, css: String) -> HistoryEntry {
        let document = parse_html(&html);
        let embedded_css = extract_embedded_css(&document);
        let css = if css.trim().is_empty() {
            embedded_css
        } else if embedded_css.trim().is_empty() {
            css
        } else {
            format!("{}\n{}", embedded_css, css)
        };
        HistoryEntry {
            url,
            html,
            css,
            document,
            focus: None,
            scroll: ScrollState { x: 0, y: 0 },
        }
    }

    fn apply_history_entry(&mut self) {
        let entry = self.history_entries[self.history_index].clone();
        self.url = entry.url;
        self.html = entry.html;
        self.css = entry.css;
        self.document = entry.document;
        self.focus = entry.focus;
        self.scroll = entry.scroll;
        self.history = self
            .history_entries
            .iter()
            .map(|entry| entry.url.clone())
            .collect();
    }

    fn persist_current_view_state(&mut self) {
        if let Some(entry) = self.history_entries.get_mut(self.history_index) {
            entry.focus = self.focus.clone();
            entry.scroll = self.scroll.clone();
        }
    }

    fn truncate_forward_history(&mut self) {
        self.persist_current_view_state();
        self.history_entries.truncate(self.history_index + 1);
    }
}

impl ConsoleEvent {
    pub fn new(level: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            level: level.into(),
            message: message.into(),
            timestamp_ms: now_ms(),
        }
    }
}

impl NetworkEvent {
    pub fn new(method: impl Into<String>, url: impl Into<String>, status: Option<u16>) -> Self {
        Self {
            method: method.into(),
            url: url.into(),
            status,
            timestamp_ms: now_ms(),
        }
    }
}

impl TraceEvent {
    pub fn new(action: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            action: action.into(),
            detail: detail.into(),
            timestamp_ms: now_ms(),
        }
    }
}

fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::browser::query_selector;

    #[test]
    fn load_html_updates_current_document_and_css() {
        let mut session = BrowserSession::new();
        session.load_html("<style>p { color: red; }</style><p id='msg'>Hi</p>");

        assert_eq!(session.url, BLANK_URL);
        assert_eq!(session.history, vec![BLANK_URL.to_string()]);
        assert!(session.css.contains("color: red"));
        assert_eq!(query_selector(&session.document, "#msg").len(), 1);
    }

    #[test]
    fn goto_back_forward_preserve_view_state() {
        let mut session = BrowserSession::new();
        session.goto_html("mem://one", "<main id='one'></main>");
        session.focus = Some("#one".to_string());
        session.scroll = ScrollState { x: 4, y: 8 };
        session.goto_html("mem://two", "<main id='two'></main>");

        assert_eq!(session.history, vec![BLANK_URL, "mem://one", "mem://two"]);
        assert!(session.back());
        assert_eq!(session.url, "mem://one");
        assert_eq!(session.focus.as_deref(), Some("#one"));
        assert_eq!(session.scroll, ScrollState { x: 4, y: 8 });
        assert!(session.forward());
        assert_eq!(session.url, "mem://two");
    }

    #[test]
    fn new_navigation_truncates_forward_history() {
        let mut session = BrowserSession::new();
        session.goto_html("mem://one", "one");
        session.goto_html("mem://two", "two");
        assert!(session.back());
        session.goto_html("mem://three", "three");

        assert_eq!(session.history, vec![BLANK_URL, "mem://one", "mem://three"]);
        assert!(!session.forward());
    }
}
