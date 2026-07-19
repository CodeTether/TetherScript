use tetherscript::browser_js::eval_with_dom;
use tetherscript::js::JsValue;

use super::model::Fixture;

pub fn assert(fixture: &Fixture) {
    let result = eval_with_dom(&fixture.html, &fixture.script)
        .unwrap_or_else(|error| panic!("{}: {error}", fixture.wpt_shape));
    assert_eq!(
        result.value,
        JsValue::String(fixture.expected_value.clone()),
        "{}: {}",
        fixture.area,
        fixture.wpt_shape
    );
}
