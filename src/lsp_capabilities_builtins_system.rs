//! Process, OS, environment, and time builtins.
//!
//! Ported from `editor/vscode/lib/tool-data-system.js`.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::lsp_capabilities::builtins_system::TABLE;
//! assert!(TABLE.iter().any(|entry| entry.0 == "env_get"));
//! ```

use crate::lsp_capabilities::builtins::Entry;

/// System builtins as `(name, params, summary)` rows.
#[rustfmt::skip]
pub const TABLE: &[Entry] = &[
    ("chdir", "path", "Change the current working directory."),
    ("cwd", "", "Return the current working directory as a Result."),
    ("env_get", "name", "Read an environment variable as a Result."),
    ("os_arch", "", "Return the operating system architecture."),
    ("os_eol", "", "Return the platform line ending."),
    ("os_homedir", "", "Return the home directory as a Result."),
    ("os_platform", "", "Return the operating system platform."),
    ("os_tmpdir", "", "Return the temporary directory."),
    ("process_arch", "", "Return the current process architecture."),
    ("process_args", "", "Return the process arguments."),
    ("process_kill", "pid[, force]", "Terminate a process and return a Result."),
    ("process_list", "", "List running processes."),
    ("process_pid", "", "Return the current process ID."),
    ("process_platform", "", "Return the current process platform."),
    ("process_run", "command[, args[, stdin[, timeout_ms]]]", "Run a subprocess."),
    ("sleep_ms", "ms", "Sleep for a number of milliseconds."),
    ("time_now_ms", "", "Return the current Unix time in milliseconds."),
];
