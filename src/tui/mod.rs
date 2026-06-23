//! Dependency-free terminal UI helpers for scripts.

mod ansi;
mod input;
mod install;
mod jsonrpc;
mod jsonrpc_error;
mod line;
mod render;
mod size;
mod stdio_err;
mod stdio_io;
mod val;
mod view;

pub(super) fn install(env: &mut crate::value::Env) {
    install::install(env);
}

#[cfg(test)]
mod stdio_tests;
#[cfg(test)]
mod tests;
