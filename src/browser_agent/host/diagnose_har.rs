//! Native HAR diagnostic serialization.

use crate::browser_agent::{BrowserHarEntry, BrowserHarTimings, BrowserPage};
use crate::value::Value;

#[path = "diagnose_har_request.rs"]
mod request;
#[path = "diagnose_har_response.rs"]
mod response;
#[cfg(test)]
#[path = "diagnose_har_tests.rs"]
mod tests;

pub(super) fn value(page: &BrowserPage) -> Value {
    super::super::super::value::list(
        page.production_debug_report()
            .network_har
            .into_iter()
            .map(entry)
            .collect(),
    )
}

fn entry(entry: BrowserHarEntry) -> Value {
    super::super::super::value::map(vec![
        ("sequence", Value::Int(entry.sequence as i64)),
        ("started_ms", Value::Int(entry.started_ms as i64)),
        ("request", request::value(entry.request)),
        ("response", response::value(entry.response)),
        ("timings", timings(entry.timings)),
    ])
}

fn timings(timings: BrowserHarTimings) -> Value {
    super::super::super::value::map(vec![
        ("total_ms", Value::Int(timings.total_ms as i64)),
        ("send_ms", Value::Int(timings.send_ms as i64)),
        ("wait_ms", Value::Int(timings.wait_ms as i64)),
        ("receive_ms", Value::Int(timings.receive_ms as i64)),
    ])
}
