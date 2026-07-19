//! Safe activation of changed file-backed controllers.

use super::state::RouteState;

pub(super) fn refresh(state: &RouteState) {
    if !state.script.source_kind.reloads() {
        return;
    }
    let _guard = state.reload_lock.lock().expect("reload lock");
    let (source, modified) = match state.script.source_kind.read() {
        Ok(source) => source,
        Err(error) => {
            state.script.reject(error.to_string(), None);
            return;
        }
    };
    if modified == state.script.modified() {
        return;
    }
    match super::validate::source(
        &state.host_factory,
        &state.plugin_name,
        &state.hook,
        &source,
    ) {
        Ok(()) => state.script.activate(source, modified),
        Err(error) => state.script.reject(error.to_string(), modified),
    }
}
