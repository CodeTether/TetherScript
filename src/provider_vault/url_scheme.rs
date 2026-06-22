//! HTTP URL scheme stripping.

use super::url::Scheme;

pub(super) fn strip(input: &str) -> Result<(Scheme, &str), String> {
    if let Some(rest) = input.strip_prefix("https://") {
        return Ok((Scheme::Https, rest));
    }
    if let Some(rest) = input.strip_prefix("http://") {
        return Ok((Scheme::Http, rest));
    }
    Err("url: expected http:// or https://".into())
}
