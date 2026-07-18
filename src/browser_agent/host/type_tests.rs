use crate::browser_agent::BrowserPage;

#[path = "type_tests_edge.rs"]
mod edge;

#[test]
fn type_appends_characters_and_dispatches_keyboard_events() {
    let mut state = super::super::state::HostState::new();
    state.page = BrowserPage::from_html("mem://type", "<input id='entry' value='a'>");
    state.page.eval_js("let log='';let n=document.getElementById('entry');n.addEventListener('keydown',function(e){log=log+'d'+e.key;});n.addEventListener('input',function(){log=log+'i';});n.addEventListener('keyup',function(e){log=log+'u'+e.key;});").unwrap();
    let payload = super::super::value::map(vec![
        ("selector", super::super::value::string("#entry")),
        ("text", super::super::value::string("bc")),
    ]);

    super::invoke(&mut state, "type", &payload).unwrap();

    assert_eq!(
        eval(&mut state, "document.getElementById('entry').value"),
        "abc"
    );
    assert_eq!(eval(&mut state, "log"), "dbiubdciuc");
}

fn payload(text: &str) -> crate::value::Value {
    super::super::value::map(vec![
        ("selector", super::super::value::string("#entry")),
        ("text", super::super::value::string(text)),
    ])
}

fn eval(state: &mut super::super::state::HostState, script: &str) -> String {
    state.page.eval_js(script).unwrap().display()
}
