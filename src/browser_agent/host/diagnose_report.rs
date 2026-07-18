//! Production-debug report serialization.

use crate::browser_agent::BrowserPage;
use crate::value::Value;

#[path = "diagnose_report_errors.rs"]
mod errors;
#[path = "diagnose_report_stacks.rs"]
mod stack;

pub(super) fn summary(page: &BrowserPage) -> Value {
    let report = page.production_debug_report();
    super::super::super::value::map(vec![
        ("url", super::super::super::value::string(report.url)),
        (
            "console_errors",
            super::value::strings(report.console_errors),
        ),
        (
            "failed_requests",
            super::value::strings(report.failed_requests),
        ),
        ("frameworks", super::value::strings(report.frameworks)),
        ("react_detected", Value::Bool(report.react.detected)),
        ("react_roots", super::value::strings(report.react.roots)),
    ])
}

pub(super) fn exceptions(page: &BrowserPage) -> Value {
    errors::exceptions(page)
}

pub(super) fn stacks(page: &BrowserPage) -> Value {
    stack::stacks(page)
}
