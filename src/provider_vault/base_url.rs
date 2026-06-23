//! Provider endpoint and API path selection.

use super::{url, url_endpoint};

pub(super) fn endpoint_path(
    provider_id: &str,
    base_url: Option<&str>,
) -> Result<(String, String), String> {
    let base = base_url
        .map(str::to_string)
        .or_else(|| inferred_base(provider_id).map(str::to_string))
        .ok_or_else(|| {
            format!("vault: provider {provider_id:?} needs base_url in Vault secrets")
        })?;
    let parsed = url::parse(base.trim_end_matches('/'))?;
    let endpoint = url_endpoint::from_url(&parsed);
    if provider_id == "openai-codex" {
        return Ok((
            endpoint,
            format!("{}/responses", parsed.path.trim_end_matches('/')),
        ));
    }
    Ok((endpoint, chat_path(&parsed.path)))
}

fn inferred_base(provider_id: &str) -> Option<&'static str> {
    match provider_id {
        "openai" => Some("https://api.openai.com/v1"),
        "openrouter" => Some("https://openrouter.ai/api/v1"),
        "openai-codex" => Some("https://chatgpt.com/backend-api/codex"),
        "cerebras" => Some("https://api.cerebras.ai/v1"),
        "zai" | "zhipuai" => Some("https://open.bigmodel.cn/api/paas/v4"),
        _ => None,
    }
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
