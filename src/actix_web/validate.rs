//! Controller source and hook validation.

use crate::plugin::PluginError;

use super::error::ActixPluginError;
use super::state::HostFactory;

pub(super) fn source(
    host_factory: &HostFactory,
    plugin_name: &str,
    hook: &str,
    source: &str,
) -> Result<(), ActixPluginError> {
    let plugin = host_factory().load_source(plugin_name, source)?;
    if plugin.has_hook(hook) {
        return Ok(());
    }
    Err(PluginError::MissingHook {
        plugin: plugin_name.into(),
        hook: hook.into(),
    }
    .into())
}
