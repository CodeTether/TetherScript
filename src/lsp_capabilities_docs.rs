//! Read-only view over the server's open-document store.
//!
//! `src/lsp.rs` keeps documents in a `HashMap<String, String>` keyed by URI and
//! replaced wholesale on `didChange` (it advertises `textDocumentSync: 1`, full
//! sync). The handlers here take that map by shared reference, so wiring them in
//! requires no change to how the server stores documents.
//!
//! A document the client has not opened is not an error: editors routinely send
//! a request for a file that was just closed. [`Docs::text`] therefore returns
//! `None` and the handler replies with a null result.
//!
//! # Examples
//!
//! ```rust
//! use std::collections::HashMap;
//! use tetherscript::lsp_capabilities::docs::Docs;
//!
//! let mut store = HashMap::new();
//! store.insert("file:///a.tether".to_string(), "let x = 1".to_string());
//! let docs = Docs::new(&store);
//! assert_eq!(docs.text("file:///a.tether"), Some("let x = 1"));
//! assert_eq!(docs.text("file:///missing.tether"), None);
//! ```

use std::collections::HashMap;
use std::path::Path;

use crate::lsp_capabilities::uri;

/// Borrowed view over open documents, keyed by URI.
pub struct Docs<'a> {
    store: &'a HashMap<String, String>,
}

impl<'a> Docs<'a> {
    /// Wrap the server's document map.
    ///
    /// # Arguments
    ///
    /// * `store` — The server's URI → text map.
    ///
    /// # Returns
    ///
    /// A read-only view.
    ///
    /// # Errors
    ///
    /// Infallible.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::collections::HashMap;
    /// use tetherscript::lsp_capabilities::docs::Docs;
    /// let store = HashMap::new();
    /// assert!(Docs::new(&store).text("file:///a.tether").is_none());
    /// ```
    pub fn new(store: &'a HashMap<String, String>) -> Self {
        Self { store }
    }

    /// Text of an open document.
    ///
    /// # Arguments
    ///
    /// * `uri` — Document URI as sent by the client.
    ///
    /// # Returns
    ///
    /// `Some(text)` when the document is open, `None` otherwise.
    ///
    /// # Errors
    ///
    /// Infallible.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::collections::HashMap;
    /// use tetherscript::lsp_capabilities::docs::Docs;
    /// let mut store = HashMap::new();
    /// store.insert("u".to_string(), "nil".to_string());
    /// assert_eq!(Docs::new(&store).text("u"), Some("nil"));
    /// ```
    pub fn text(&self, uri: &str) -> Option<&'a str> {
        self.store.get(uri).map(String::as_str)
    }

    /// Text of an imported module, preferring the open buffer over disk.
    ///
    /// An unsaved edit in another tab is the state the user can see, so the
    /// buffer wins; falling back to disk is what makes cross-module navigation
    /// work for files the user has never opened.
    ///
    /// # Arguments
    ///
    /// * `path` — Canonical path of the imported module.
    ///
    /// # Returns
    ///
    /// `Some(text)` from the open buffer or from disk, `None` when the file
    /// cannot be read.
    ///
    /// # Errors
    ///
    /// Infallible; an unreadable file is reported as `None`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::collections::HashMap;
    /// use std::path::Path;
    /// use tetherscript::lsp_capabilities::docs::Docs;
    /// let store = HashMap::new();
    /// assert!(Docs::new(&store).module_text(Path::new("/no/such.tether")).is_none());
    /// ```
    pub fn module_text(&self, path: &Path) -> Option<String> {
        let target = uri::from_path(path);
        if let Some(open) = self.store.get(&target) {
            return Some(open.clone());
        }
        std::fs::read_to_string(path).ok()
    }
}
