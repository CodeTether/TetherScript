//! HAR request field serialization.

use crate::browser_agent::BrowserHarRequest;
use crate::value::Value;

pub(super) fn value(request: BrowserHarRequest) -> Value {
    super::super::super::super::value::map(vec![
        (
            "method",
            super::super::super::super::value::string(request.method),
        ),
        (
            "url",
            super::super::super::super::value::string(request.url),
        ),
        ("headers", super::headers::value(request.headers)),
        (
            "post_data",
            request
                .post_data
                .map_or(Value::Nil, super::super::super::super::value::string),
        ),
    ])
}
