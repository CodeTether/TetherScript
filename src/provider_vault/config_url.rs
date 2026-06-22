//! Vault KV v2 URL construction.

use super::config::VaultConfig;

pub(super) fn secret_url(config: &VaultConfig, provider_id: &str) -> String {
    format!(
        "{}/v1/{}/data/{}/{}",
        config.address,
        clean(&config.mount),
        clean(&config.path),
        clean(provider_id)
    )
}

pub(super) fn list_url(config: &VaultConfig) -> String {
    format!(
        "{}/v1/{}/metadata/{}?list=true",
        config.address,
        clean(&config.mount),
        clean(&config.path)
    )
}

fn clean(value: &str) -> &str {
    value.trim_matches('/')
}
