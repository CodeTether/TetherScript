//! HAR header-pair serialization.

use crate::value::Value;

pub(super) fn value(headers: Vec<(String, String)>) -> Value {
    super::super::super::super::value::list(
        headers
            .into_iter()
            .map(|(name, value)| {
                super::super::super::super::value::map(vec![
                    ("name", super::super::super::super::value::string(name)),
                    ("value", super::super::super::super::value::string(value)),
                ])
            })
            .collect(),
    )
}
