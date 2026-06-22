//! Vault environment configuration.

use std::env;

pub(super) struct VaultConfig {
    address: String,
    pub token: String,
    mount: String,
    path: String,
}

impl VaultConfig {
    pub fn from_env() -> Result<Self, String> {
        Ok(Self {
            address: required("VAULT_ADDR")?.trim_end_matches('/').to_string(),
            token: required("VAULT_TOKEN")?,
            mount: optional("VAULT_MOUNT", "secret"),
            path: optional("VAULT_SECRETS_PATH", "codetether/providers"),
        })
    }

    pub fn secret_url(&self, provider_id: &str) -> String {
        format!(
            "{}/v1/{}/data/{}/{}",
            self.address,
            clean(&self.mount),
            clean(&self.path),
            clean(provider_id)
        )
    }
}

fn required(name: &str) -> Result<String, String> {
    env::var(name)
        .map(|value| value.trim().to_string())
        .map_err(|_| format!("vault: {name} is not set"))
        .and_then(|value| {
            if value.is_empty() {
                Err(format!("vault: {name} is empty"))
            } else {
                Ok(value)
            }
        })
}

fn optional(name: &str, default: &str) -> String {
    env::var(name).unwrap_or_else(|_| default.into())
}

fn clean(value: &str) -> &str {
    value.trim_matches('/')
}
