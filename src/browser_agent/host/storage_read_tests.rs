use crate::value::Value;

#[test]
fn reads_actual_origin_storage_records() {
    let mut state = super::super::super::super::state::HostState::new();
    state.page.goto_html("https://app.test/page", "");
    state.page.set_local_storage_item("theme", "dark");
    state.page.set_session_storage_item("draft", "yes");
    for (action, key, expected) in [
        ("local_storage", "theme", "dark"),
        ("session_storage", "draft", "yes"),
    ] {
        let Value::Map(values) = super::invoke(&mut state, action).unwrap() else {
            panic!("expected storage record map");
        };
        assert_eq!(
            values.borrow().get(key),
            Some(&super::super::super::super::value::string(expected))
        );
    }
}
