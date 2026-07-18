//! Live HTTP loading for native top-level navigation.

const MAX_REDIRECTS: usize = 20;

pub(super) struct LoadedPage {
    pub(super) url: String,
    pub(super) body: String,
}

pub(super) fn load(url: &str) -> Result<LoadedPage, String> {
    let mut current = url.to_string();
    for _ in 0..=MAX_REDIRECTS {
        let response = crate::http::client_request("GET", &current, None, &[])?;
        let parts = super::fetch_response::parts(response)?;
        if matches!(parts.status, 301 | 302 | 303 | 307 | 308) {
            let target = parts.location.ok_or_else(|| {
                format!("browser.goto: redirect from {} has no location", current)
            })?;
            current = super::url::resolve(&current, &target);
            continue;
        }
        if (200..300).contains(&parts.status) {
            return Ok(LoadedPage {
                url: current,
                body: parts.body,
            });
        }
        return Err(format!(
            "browser.goto: {} returned HTTP {}",
            current, parts.status
        ));
    }
    Err(format!("browser.goto: redirect limit exceeded for {}", url))
}
