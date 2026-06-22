//! Default Vault provider selection for full access mode.

use std::rc::Rc;

use crate::capability::Authority;

pub(super) fn load() -> Result<Option<Rc<dyn Authority>>, String> {
    if let Some(id) = env_provider() {
        return super::load(&id).map(Some);
    }
    let config = match super::config::VaultConfig::from_env() {
        Ok(config) => config,
        Err(_) => return Ok(None),
    };
    let body = super::http::get(&super::config_url::list_url(&config), &config.token)?;
    let root = crate::json::parse_str(&body)
        .map_err(|error| format!("vault: invalid provider list JSON: {error}"))?;
    let ids = super::list::provider_ids(&root)?;
    match preferred(&ids) {
        Some(id) => super::load(id).map(Some),
        None => Ok(None),
    }
}

fn env_provider() -> Option<String> {
    ["TETHERSCRIPT_PROVIDER", "TETHERSCRIPT_AGENT_PROVIDER"]
        .into_iter()
        .find_map(|name| std::env::var(name).ok())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn preferred(ids: &[String]) -> Option<&str> {
    let order = ["openai", "openrouter", "cerebras", "zai", "zhipuai"];
    order
        .iter()
        .find(|id| ids.iter().any(|candidate| candidate == **id))
        .copied()
        .or_else(|| ids.first().map(String::as_str))
}
