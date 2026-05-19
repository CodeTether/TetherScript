use tetherscript::browser_js::eval_with_dom;
use tetherscript::js::JsValue;

pub struct Case {
    pub area: &'static str,
    pub wpt_shape: &'static str,
    pub unsupported: &'static [&'static str],
}

pub struct DomCase {
    pub case: Case,
    pub html: &'static str,
    pub script: &'static str,
    pub expect: &'static str,
}

pub fn assert_case(case: &Case) {
    assert!(!case.area.is_empty(), "fixture area is required");
    assert!(!case.wpt_shape.is_empty(), "fixture WPT shape is required");
    assert!(
        !case.unsupported.is_empty(),
        "fixture family must document unsupported behavior"
    );
}

pub fn assert_dom(fixture: &DomCase) {
    assert_case(&fixture.case);
    let result = eval_with_dom(fixture.html, fixture.script).unwrap().value;
    assert_eq!(
        result,
        JsValue::String(fixture.expect.into()),
        "{}: {}",
        fixture.case.area,
        fixture.case.wpt_shape
    );
}
