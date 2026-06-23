//! Provider authority construction from Vault secrets.

use std::rc::Rc;

use crate::capability::Authority;
use crate::provider_cap::ProviderAuthority;

use super::{base_url, secret::ProviderSecret};

pub(super) fn build(
    provider_id: &str,
    secret: ProviderSecret,
) -> Result<Rc<dyn Authority>, String> {
    let (endpoint, path) = base_url::endpoint_path(provider_id, secret.base_url.as_deref())?;
    let mut auth = ProviderAuthority::new(&endpoint);
    auth = ProviderAuthority::with_path(auth, &path);
    if provider_id == "openai-codex" {
        auth = bind_codex(auth, &secret)?;
    } else if let Some(key) = secret.api_key {
        auth =
            ProviderAuthority::with_bound_header(auth, "Authorization", &format!("Bearer {key}"));
    }
    if let Some(org) = secret.organization {
        auth = ProviderAuthority::with_bound_header(auth, "OpenAI-Organization", &org);
    }
    for (name, value) in secret.headers {
        auth = ProviderAuthority::with_bound_header(auth, &name, &value);
    }
    Ok(auth)
}

fn bind_codex(
    mut auth: Rc<dyn Authority>,
    secret: &ProviderSecret,
) -> Result<Rc<dyn Authority>, String> {
    let token = secret
        .access_token
        .as_ref()
        .or(secret.api_key.as_ref())
        .ok_or("vault: openai-codex requires access_token")?;
    auth = ProviderAuthority::with_bound_header(auth, "Authorization", &format!("Bearer {token}"));
    if let Some(id) = secret
        .chatgpt_account_id
        .as_ref()
        .or(secret.organization.as_ref())
    {
        auth = ProviderAuthority::with_bound_header(auth, "chatgpt-account-id", id);
    }
    Ok(auth)
}
