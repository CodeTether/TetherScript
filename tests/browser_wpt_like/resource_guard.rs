use super::case::{assert_case, Case};
use tetherscript::browser_agent::{BrowserPage, BrowserResourceLimits};

const CASE: Case = Case {
    area: "browser/resource-limits",
    wpt_shape: "resource guards expose quota estimates, reject oversized storage writes, and report memory-pressure cache trimming",
    unsupported: &["real operating-system memory pressure signals"],
};

pub fn run() {
    assert_case(&CASE);
    dom_and_trace_guard_metadata();
    storage_manager_estimate_returns_usage_and_quota();
    storage_write_over_quota_fails_with_quota_exceeded_error();
    memory_pressure_simulation_reports_cache_status();
}

fn dom_and_trace_guard_metadata() {
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

fn storage_manager_estimate_returns_usage_and_quota() {
    let mut page = BrowserPage::from_html("mem://quota-estimate", "<main></main>");
    page.eval_js("localStorage.setItem('a','12345'); sessionStorage.setItem('b','xy')")
        .unwrap();

    let value = page
        .eval_js("let out='';navigator.storage.estimate().then(function(e){out=e.quota + ':' + e.usage;});out")
        .unwrap();

    assert_eq!(value.display(), "67108864:0");
}

fn storage_write_over_quota_fails_with_quota_exceeded_error() {
    let mut page = BrowserPage::from_html("mem://quota-exceeded", "<main></main>");
    let err = page
        .eval_js("localStorage.setItem('blob','xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx')")
        .unwrap_err();

    assert!(
        err.contains("QuotaExceededError"),
        "expected quota exceeded error, got: {err}"
    );
}

fn memory_pressure_simulation_reports_cache_status() {
    let mut page = BrowserPage::from_html("mem://memory-pressure", "<main></main>");
    page.eval_js("window.__agentEssentialCache={keep:true};window.__agentNonEssentialCache={blob:'xxxxxxxx'}")
        .unwrap();

    let status = page
        .simulate_memory_pressure()
        .expect("memory pressure simulation should succeed");

    assert_eq!(status.dropped_non_essential_caches, 1);
    assert!(status.essential_caches_retained);
}
