use std::cell::RefCell;
use std::rc::Rc;

use crate::browser_agent::BrowserPage;
use crate::value::Value;

#[test]
fn upload_reads_paths_and_populates_a_file_input() {
    let mut state = super::super::state::HostState::new();
    state.page = BrowserPage::from_html("mem://upload", "<input id='upload' type='file'>");
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/examples/hello.tether");
    let payload = super::super::value::map(vec![
        ("selector", super::super::value::string("#upload")),
        (
            "paths",
            Value::List(Rc::new(RefCell::new(vec![super::super::value::string(
                path,
            )]))),
        ),
    ]);

    super::invoke(&mut state, "upload", &payload).unwrap();

    let actual = state
        .page
        .eval_js("let f=document.getElementById('upload').files[0];f.name+':'+f.type+':'+f.size;")
        .unwrap()
        .display();
    let size = std::fs::metadata(path).unwrap().len();
    assert_eq!(actual, format!("hello.tether:text/plain:{size}"));
}
