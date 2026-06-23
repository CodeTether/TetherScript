//! Vault KV v2 provider capability loading.

mod authority;
mod base_url;
mod body;
mod config;
mod config_url;
mod default;
mod fields;
mod http;
mod list;
mod request;
mod response;
mod secret;
pub(crate) mod url;
pub(crate) mod url_endpoint;
mod url_scheme;

use std::rc::Rc;

use crate::capability::Authority;

pub(crate) fn load(provider_id: &str) -> Result<Rc<dyn Authority>, String> {
    let config = config::VaultConfig::from_env()?;
    let body = http::get(&config_url::secret_url(&config, provider_id), &config.token)?;
    let root = crate::json::parse_str(&body)
        .map_err(|error| format!("vault: invalid JSON response: {error}"))?;
    authority::build(provider_id, secret::parse(&root)?)
}

pub(crate) fn load_default() -> Result<Option<Rc<dyn Authority>>, String> {
    default::load()
}

#[cfg(test)]
mod body_tests;
#[cfg(test)]
mod tests;
