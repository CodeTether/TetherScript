//! Tetherscript values for captured network events.

use crate::browser_session::NetworkEvent;
use crate::value::Value;

pub(super) fn from_event(event: &NetworkEvent) -> Value {
    super::super::super::value::map(vec![
        ("method", super::super::super::value::string(&event.method)),
        ("url", super::super::super::value::string(&event.url)),
        (
            "status",
            event
                .status
                .map_or(Value::Nil, |status| Value::Int(status.into())),
        ),
        (
            "route_result",
            event
                .route_result
                .as_ref()
                .map_or(Value::Nil, super::super::super::value::string),
        ),
    ])
}
