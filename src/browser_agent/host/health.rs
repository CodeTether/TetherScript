//! Native browser host health response.

use crate::value::Value;

use super::state::HostState;

pub(super) fn value(state: &HostState) -> Value {
    super::value::map(vec![
        ("backend", super::value::string("tetherscript-native")),
        ("started", Value::Bool(state.started)),
        ("url", super::value::string(state.page.session.url.clone())),
    ])
}
