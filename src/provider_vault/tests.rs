use super::{base_url, secret};
use crate::json;

#[test]
fn extracts_codetether_provider_secret() {
    let raw = r#"{"data":{"data":{"api_key":"sk","base_url":"https://x/v1"}}}"#;
    let root = json::parse_str(raw).unwrap();
    let secret = secret::parse(&root).unwrap();
    assert_eq!(secret.api_key.as_deref(), Some("sk"));
    assert_eq!(secret.base_url.as_deref(), Some("https://x/v1"));
}

#[test]
fn maps_base_url_to_chat_completion_path() {
    let (endpoint, path) =
        base_url::endpoint_path("openai", Some("https://api.openai.com/v1")).unwrap();
    assert_eq!(endpoint, "https://api.openai.com");
    assert_eq!(path, "/v1/chat/completions");
}
