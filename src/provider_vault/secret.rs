//! CodeTether-compatible provider secret shape.

use super::fields;
use crate::value::Value;

pub(super) struct ProviderSecret {
    pub api_key: Option<String>,
    pub access_token: Option<String>,
    pub base_url: Option<String>,
    pub chatgpt_account_id: Option<String>,
    pub organization: Option<String>,
    pub headers: Vec<(String, String)>,
}

pub(super) fn parse(root: &Value) -> Result<ProviderSecret, String> {
    let secret = fields::kv2_secret(root)?;
    let Value::Map(map) = secret else {
        return Err("vault: data.data must be a map".into());
    };
    let map = map.borrow();
    Ok(ProviderSecret {
        api_key: fields::string(&map, "api_key"),
        access_token: fields::string(&map, "access_token"),
        base_url: fields::string(&map, "base_url"),
        chatgpt_account_id: fields::string(&map, "chatgpt_account_id"),
        organization: fields::string(&map, "organization"),
        headers: fields::headers(&map),
    })
}
