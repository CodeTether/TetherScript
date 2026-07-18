//! Upload-path extraction from native action envelopes.

use crate::value::Value;

pub(super) fn parse(payload: &Value) -> Result<Vec<String>, String> {
    match super::super::super::value::field(payload, "paths")? {
        Value::Str(path) => Ok(vec![(*path).clone()]),
        Value::List(paths) => paths
            .borrow()
            .iter()
            .enumerate()
            .map(|(index, path)| match path {
                Value::Str(path) => Ok((**path).clone()),
                value => Err(format!(
                    "browser.upload: path {} must be str, got {}",
                    index,
                    value.type_name()
                )),
            })
            .collect(),
        value => Err(format!(
            "browser.upload: `paths` must be str or list, got {}",
            value.type_name()
        )),
    }
}
