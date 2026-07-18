use crate::browser_agent::{BrowserPage, Locator};

#[test]
fn keyboard_type_appends_to_the_focused_element() {
    let mut state = super::super::state::HostState::new();
    state.page = BrowserPage::from_html("mem://keyboard-type", "<input id='entry' value='a'>");
    state.focused = Some(Locator::css("#entry"));
    state.page.eval_js("let log='';let n=document.getElementById('entry');n.addEventListener('keydown',function(e){log=log+'d'+e.key;});n.addEventListener('input',function(){log=log+'i';});n.addEventListener('keyup',function(e){log=log+'u'+e.key;});").unwrap();
    let payload = super::super::value::map(vec![("text", super::super::value::string("bc"))]);

    super::invoke(&mut state, "keyboard_type", &payload).unwrap();

    assert_eq!(
        eval(&mut state, "document.getElementById('entry').value"),
        "abc"
    );
    assert_eq!(eval(&mut state, "log"), "dbiubdciuc");
}

#[test]
fn keyboard_type_requires_host_focus() {
    let mut state = super::super::state::HostState::new();
    let payload = super::super::value::map(vec![("text", super::super::value::string("x"))]);
    let error = super::invoke(&mut state, "keyboard_type", &payload).unwrap_err();
    assert!(error.contains("browser.keyboard_type: no focused element"));
}

fn eval(state: &mut super::super::state::HostState, script: &str) -> String {
    state.page.eval_js(script).unwrap().display()
}
