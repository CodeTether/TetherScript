//! Provider authority construction from environment configuration.

use std::rc::Rc;

use crate::capability::Authority;
use crate::provider_cap::ProviderAuthority;

use super::{base, candidate::Candidate, vars};

pub(super) fn from_candidate(candidate: &Candidate) -> Result<Rc<dyn Authority>, String> {
    let key = vars::first(candidate.key_vars).ok_or_else(|| {
        format!(
            "provider env: {} requires one of {:?}",
            candidate.id, candidate.key_vars
        )
    })?;
    let base = vars::first(candidate.base_vars).unwrap_or_else(|| candidate.default_base.into());
    direct(
        &base,
        Some(&key),
        vars::first(candidate.org_vars).as_deref(),
    )
}

pub(super) fn direct(
    base_url: &str,
    key: Option<&str>,
    org: Option<&str>,
) -> Result<Rc<dyn Authority>, String> {
    let (endpoint, path) = base::endpoint_path(base_url)?;
    let mut auth = ProviderAuthority::new(&endpoint);
    auth = ProviderAuthority::with_path(auth, &path);
    if let Some(key) = key {
        auth =
            ProviderAuthority::with_bound_header(auth, "Authorization", &format!("Bearer {key}"));
    }
    if let Some(org) = org {
        auth = ProviderAuthority::with_bound_header(auth, "OpenAI-Organization", org);
    }
    Ok(auth)
}
