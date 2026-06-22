//! Vault KV v2 provider capability loading.

mod authority;
mod base_url;
mod config;
mod fields;
mod http;
mod request;
mod response;
mod secret;
mod url;
mod url_endpoint;
mod url_scheme;

use std::rc::Rc;

use crate::capability::Authority;

pub(crate) fn load(provider_id: &str) -> Result<Rc<dyn Authority>, String> {
    let config = config::VaultConfig::from_env()?;
    let body = http::get(&config.secret_url(provider_id), &config.token)?;
    let root = crate::json::parse_str(&body)
        .map_err(|error| format!("vault: invalid JSON response: {error}"))?;
    authority::build(provider_id, secret::parse(&root)?)
}

#[cfg(test)]
mod tests;
