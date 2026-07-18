//! Tetherscript tab-list values.

use std::cell::RefCell;
use std::rc::Rc;

use crate::value::Value;

use super::super::super::state::HostState;

pub(super) fn list(state: &HostState) -> Value {
    let values = state
        .tabs
        .iter()
        .enumerate()
        .map(|(index, page)| {
            let page = page.as_ref().unwrap_or(&state.page);
            super::super::super::value::map(vec![
                ("index", Value::Int(index as i64)),
                ("url", super::super::super::value::string(&page.session.url)),
                ("active", Value::Bool(index == state.active_tab)),
            ])
        })
        .collect();
    Value::List(Rc::new(RefCell::new(values)))
}
