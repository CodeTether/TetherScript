//! OpenAI-compatible base URL normalization.

use crate::provider_vault::{url, url_endpoint};

pub(super) fn endpoint_path(base: &str) -> Result<(String, String), String> {
    let parsed = url::parse(base.trim_end_matches('/'))?;
    Ok((url_endpoint::from_url(&parsed), chat_path(&parsed.path)))
}

fn chat_path(path: &str) -> String {
    let path = path.trim_end_matches('/');
    if path.ends_with("/chat/completions") {
        path.into()
    } else if path.is_empty() || path == "/" {
        "/v1/chat/completions".into()
    } else {
        format!("{path}/chat/completions")
    }
}
