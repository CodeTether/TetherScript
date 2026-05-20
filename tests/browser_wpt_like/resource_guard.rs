use super::case::{assert_case, Case};
use tetherscript::browser_agent::{BrowserPage, BrowserResourceLimits};

const CASE: Case = Case {
    area: "browser/resource-limits",
    wpt_shape: "resource-limit guard rejects oversized DOM and enforces trace caps",
    unsupported: &["per-origin network budgets", "real memory pressure signals"],
};

pub fn run() {
    assert_case(&CASE);
    let mut page = BrowserPage::from_html("mem://limits", "<p>ok</p>");
    let tiny = BrowserResourceLimits {
        max_dom_bytes: 4,
        max_action_attempts: 1,
        max_action_ticks: 1,
        max_trace_entries: 1,
        max_raster_pixels: 4,
    };
    page.set_resource_limits(tiny);
    assert_eq!(page.resource_limits(), tiny);
    let meta = page.guard_metadata();
    assert_eq!(meta.max_dom_bytes, 4);
    assert_eq!(meta.max_trace_entries, 1);
    let oversize = page.run_scripts();
    assert!(oversize.is_err(), "oversize DOM should fail resource guard");
}
