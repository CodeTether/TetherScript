use crate::browser_agent::BrowserPage;
use crate::value::Value;

#[test]
fn exports_native_request_and_response_records() {
    let mut page = BrowserPage::from_html(
        "https://app.test/",
        "<script>fetch('/api',{method:'POST',body:'x'});</script>",
    );
    page.run_scripts().unwrap();
    let Value::List(entries) = super::value(&page) else {
        panic!("expected HAR entry list");
    };
    let entries = entries.borrow();
    let Value::Map(entry) = &entries[0] else {
        panic!("expected HAR entry map");
    };
    assert!(matches!(entry.borrow().get("request"), Some(Value::Map(_))));
    assert!(matches!(
        entry.borrow().get("response"),
        Some(Value::Map(_))
    ));
}
