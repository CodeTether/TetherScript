//! Provider capability construction for CLI grants.

use std::rc::Rc;

use crate::capability::Authority;
use crate::{provider_cap, provider_vault};

pub(super) fn authority(
    endpoint: &Option<String>,
    key: &Option<String>,
    vault_id: &Option<String>,
) -> Result<Option<Rc<dyn Authority>>, String> {
    match (endpoint, vault_id) {
        (Some(_), Some(_)) => {
            Err("use either --grant-provider or --grant-provider-vault, not both".into())
        }
        (None, Some(_)) if key.is_some() => {
            Err("--grant-provider-key is only valid with --grant-provider".into())
        }
        (None, Some(id)) => provider_vault::load(id).map(Some),
        (Some(endpoint), None) => Ok(Some(direct(endpoint, key))),
        (None, None) => Ok(None),
    }
}

fn direct(endpoint: &str, key: &Option<String>) -> Rc<dyn Authority> {
    let auth = provider_cap::ProviderAuthority::new(endpoint);
    if let Some(key) = key {
        provider_cap::ProviderAuthority::with_bound_header(
            auth,
            "Authorization",
            &format!("Bearer {key}"),
        )
    } else {
        auth
    }
}
