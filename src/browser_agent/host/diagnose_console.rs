//! Console event diagnostic views.

use crate::browser_agent::BrowserPage;
use crate::value::Value;

pub(super) fn logs(page: &BrowserPage) -> Value {
    super::value::list(
        page.console_events()
            .iter()
            .map(|event| {
                super::super::super::value::map(vec![
                    ("level", super::super::super::value::string(&event.level)),
                    (
                        "message",
                        super::super::super::value::string(&event.message),
                    ),
                    (
                        "timestamp_ms",
                        Value::Int(i64::try_from(event.timestamp_ms).unwrap_or(i64::MAX)),
                    ),
                ])
            })
            .collect(),
    )
}

pub(super) fn errors(page: &BrowserPage) -> Value {
    super::value::strings(
        page.console_events()
            .iter()
            .filter(|event| event.level == "error")
            .map(|event| event.message.clone()),
    )
}

pub(super) fn rejections(page: &BrowserPage) -> Value {
    super::value::strings(page.console_events().iter().filter_map(|event| {
        let lower = event.message.to_ascii_lowercase();
        (lower.contains("unhandled") || lower.contains("rejection")).then(|| event.message.clone())
    }))
}
