//! Per-blocking-thread plugin cache keyed by script generation.

use std::cell::RefCell;
use std::collections::HashMap;

use crate::plugin::{LoadedPlugin, PluginCall, PluginError};
use crate::value::Value;

use super::state::RouteState;

struct CachedPlugin {
    generation: u64,
    plugin: LoadedPlugin,
}

thread_local! {
    static PLUGINS: RefCell<HashMap<usize, CachedPlugin>> = RefCell::new(HashMap::new());
}

pub(super) fn call(state: &RouteState, arguments: &[Value]) -> Result<PluginCall, PluginError> {
    let snapshot = state.script.snapshot();
    PLUGINS.with(|plugins| {
        let mut plugins = plugins.borrow_mut();
        let stale = plugins
            .get(&state.cache_key)
            .map(|cached| cached.generation != snapshot.generation)
            .unwrap_or(true);
        if stale {
            let plugin = state.load(&snapshot.source)?;
            plugins.insert(
                state.cache_key,
                CachedPlugin {
                    generation: snapshot.generation,
                    plugin,
                },
            );
        }
        plugins
            .get_mut(&state.cache_key)
            .expect("plugin cache entry")
            .plugin
            .call(&state.hook, arguments)
    })
}
