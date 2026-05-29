//! Describe values for computer authorities.

use crate::value::Value;

use super::authority::ComputerAuthority;

impl ComputerAuthority {
    pub(crate) fn describe(&self) -> Value {
        super::value::map_value(vec![
            ("kind", super::value::str_value("computer")),
            ("endpoint", super::value::str_value(self.endpoint.clone())),
            ("scopes", self.scope_map()),
            ("origin", self.origin_value()),
            (
                "mdns_service_type",
                super::value::str_value("_codetether-computer._tcp.local."),
            ),
        ])
    }

    fn scope_map(&self) -> Value {
        super::value::owned_map(
            self.allowed_scopes
                .iter()
                .map(|s| (s.clone(), Value::Bool(true)))
                .collect(),
        )
    }

    fn origin_value(&self) -> Value {
        super::value::str_value(self.origin.clone().unwrap_or_default())
    }
}
