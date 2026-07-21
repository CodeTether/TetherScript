//! Shared-view native rendering coverage.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::value::resource::OwnedResource;
use crate::value::Value;

use super::ok;

#[test]
fn terminal_view_schema_renders_to_an_rgba_surface() {
    let mut fields = HashMap::new();
    fields.insert("title".into(), Value::Str(Rc::new("dual UI".into())));
    fields.insert("status".into(), Value::Str(Rc::new("ready".into())));
    fields.insert(
        "items".into(),
        Value::List(Rc::new(RefCell::new(vec![Value::Str(Rc::new(
            "same view".into(),
        ))]))),
    );
    let view = Value::Map(Rc::new(RefCell::new(fields)));
    let mut surface = OwnedResource::render_surface(640, 384, 1, 245_760).unwrap();
    assert_eq!(
        ok(surface.call("render_view", &[view]).unwrap()),
        Value::Int(245_760)
    );
}
