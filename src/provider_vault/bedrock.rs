//! Bedrock provider construction from Vault secrets.

use std::rc::Rc;

use crate::capability::Authority;
use crate::provider_bedrock::BedrockAuthority;

use super::secret::ProviderSecret;

pub(super) fn build(secret: ProviderSecret) -> Result<Rc<dyn Authority>, String> {
    let region = secret
        .region
        .clone()
        .or_else(|| secret.headers.iter().find(|(k, _)| k == "region").map(|(_, v)| v.clone()))
        .or_else(|| std::env::var("AWS_REGION").ok())
        .or_else(|| std::env::var("AWS_DEFAULT_REGION").ok())
        .unwrap_or_else(|| "us-east-1".to_string());
    let access_key = secret
        .aws_access_key_id
        .or(secret.api_key)
        .ok_or("vault: bedrock requires aws_access_key_id")?;
    let secret_key = secret
        .aws_secret_access_key
        .ok_or("vault: bedrock requires aws_secret_access_key")?;
    Ok(BedrockAuthority::new(
        access_key,
        secret_key,
        secret.aws_session_token,
        region,
    ))
}
