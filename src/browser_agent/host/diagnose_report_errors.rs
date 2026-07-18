//! Classified runtime-exception serialization.

use crate::browser_agent::{BrowserPage, RuntimeExceptionKind};
use crate::value::Value;

pub(super) fn exceptions(page: &BrowserPage) -> Value {
    super::super::value::list(
        page.production_debug_report()
            .runtime_exceptions
            .into_iter()
            .map(|error| {
                super::super::super::super::value::map(vec![
                    (
                        "action",
                        super::super::super::super::value::string(error.action),
                    ),
                    (
                        "message",
                        super::super::super::super::value::string(error.message),
                    ),
                    (
                        "kind",
                        super::super::super::super::value::string(kind(error.kind)),
                    ),
                ])
            })
            .collect(),
    )
}

fn kind(kind: RuntimeExceptionKind) -> &'static str {
    match kind {
        RuntimeExceptionKind::Reference => "reference",
        RuntimeExceptionKind::Type => "type",
        RuntimeExceptionKind::Syntax => "syntax",
        RuntimeExceptionKind::Network => "network",
        RuntimeExceptionKind::Cors => "cors",
        RuntimeExceptionKind::Abort => "abort",
        RuntimeExceptionKind::Permission => "permission",
        RuntimeExceptionKind::Other => "other",
    }
}
