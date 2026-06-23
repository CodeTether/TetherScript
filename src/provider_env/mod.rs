//! Provider capability loading from local development environment variables.

mod authority;
mod base;
mod candidate;
mod vars;

use std::rc::Rc;

use crate::capability::Authority;

pub(crate) fn load_default() -> Result<Option<Rc<dyn Authority>>, String> {
    if let Some(endpoint) = vars::get("TETHERSCRIPT_PROVIDER_ENDPOINT") {
        return authority::direct(
            &endpoint,
            vars::get("TETHERSCRIPT_PROVIDER_KEY").as_deref(),
            None,
        )
        .map(Some);
    }
    if let Some(id) = selected_provider() {
        return candidate::named(&id)
            .ok_or_else(|| format!("provider env: unsupported provider {id:?}"))
            .and_then(authority::from_candidate)
            .map(Some);
    }
    candidate::first_configured()
        .map(authority::from_candidate)
        .transpose()
}

pub(crate) fn fallback_enabled() -> bool {
    !matches!(
        vars::get("CODETETHER_DISABLE_ENV_FALLBACK").as_deref(),
        Some("1")
    )
}

fn selected_provider() -> Option<String> {
    vars::get("TETHERSCRIPT_PROVIDER").or_else(|| vars::get("TETHERSCRIPT_AGENT_PROVIDER"))
}
