use crate::value::Value;

#[test]
fn summary_returns_origin_scoped_native_records() {
    let mut state = super::super::super::state::HostState::new();
    state.page.goto_html("https://app.test/page", "");
    state
        .page
        .indexed_db_put("app", "settings", "theme", "dark")
        .unwrap();
    let Value::List(records) = super::invoke(&state).unwrap() else {
        panic!("expected IndexedDB record list");
    };
    let records = records.borrow();
    let Value::Map(record) = &records[0] else {
        panic!("expected record map");
    };
    assert_eq!(
        record.borrow().get("database"),
        Some(&super::super::super::value::string("app"))
    );
    assert_eq!(
        record.borrow().get("value"),
        Some(&super::super::super::value::string("dark"))
    );
}
