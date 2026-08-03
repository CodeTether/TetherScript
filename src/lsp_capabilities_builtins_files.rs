//! Filesystem and path builtins.
//!
//! Ported from `editor/vscode/lib/tool-data-files.js`. Note that the ambient
//! `fs_*` builtins listed here bypass capability grants (see AGENTS.md, "What's
//! not done yet"); the catalog documents what exists rather than what should.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::lsp_capabilities::builtins_files::TABLE;
//! assert!(TABLE.iter().any(|entry| entry.0 == "fs_read"));
//! ```

use crate::lsp_capabilities::builtins::Entry;

/// Filesystem and path builtins as `(name, params, summary)` rows.
#[rustfmt::skip]
pub const TABLE: &[Entry] = &[
    ("fs_copy", "from, to", "Copy a filesystem entry."),
    ("fs_exists", "path", "Return whether a path exists."),
    ("fs_list", "path", "List directory entries."),
    ("fs_mkdir", "path", "Create a directory."),
    ("fs_read", "path", "Read a UTF-8 file."),
    ("fs_remove", "path", "Remove a filesystem entry."),
    ("fs_rename", "from, to", "Rename a filesystem entry."),
    ("fs_stat", "path", "Return filesystem metadata."),
    ("fs_write", "path, body", "Write a UTF-8 file."),
    ("path_basename", "path", "Return the final path component."),
    ("path_dirname", "path", "Return the parent directory."),
    ("path_extname", "path", "Return the filename extension."),
    ("path_join", "parts", "Join a list of path components."),
    ("path_normalize", "path", "Normalize path components."),
    ("path_resolve", "path", "Resolve a path from the working directory."),
    ("path_sep", "", "Return the platform path separator."),
];
