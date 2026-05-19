use super::case::{assert_case, Case};
use tetherscript::browser_agent::BrowserPage;

const CASE: Case = Case {
    area: "fetch/cors/mixed-content/origin-policy",
    wpt_shape: "same-origin requests are allowed and cross-origin allowlist is explicit",
    unsupported: &["full CSP parser", "COOP/COEP and mixed-content enforcement"],
};

pub fn run() {
    assert_case(&CASE);
    let mut page = BrowserPage::from_html("https://app.test/path#view", "");
    let same = page.request_security_metadata("/api");
    let cross = page.request_security_metadata("https://api.test/data");
    assert!(same.same_origin);
    assert!(same.allowed_by_policy);
    assert_eq!(cross.referrer.as_deref(), Some("https://app.test/path"));
    assert!(!cross.allowed_by_policy);
    page.allow_origin("https://api.test");
    assert!(page.is_request_allowed("https://api.test/v1"));
}
