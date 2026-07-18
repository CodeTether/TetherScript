use super::super::*;

#[path = "commands/clipboard.rs"]
mod clipboard;
#[path = "commands/command.rs"]
mod command;
#[path = "commands/edit.rs"]
mod edit;
#[path = "commands/execute.rs"]
mod execute;
#[path = "commands/install.rs"]
mod install;
#[path = "commands/query.rs"]
mod query;
#[path = "commands/state.rs"]
mod state;

#[cfg(test)]
#[path = "commands/tests.rs"]
mod tests;

pub(in crate::browser_js) fn install(object: &mut HashMap<String, JsValue>, document: &DomHandle) {
    install::methods(object, document);
}

pub(in crate::browser_js) fn reset() {
    state::reset();
}
