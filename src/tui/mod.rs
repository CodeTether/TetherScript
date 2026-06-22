//! Dependency-free terminal UI helpers for scripts.

mod ansi;
mod input;
mod install;
mod line;
mod render;
mod size;
mod val;
mod view;

pub(super) fn install(env: &mut crate::value::Env) {
    install::install(env);
}

#[cfg(test)]
mod tests;
