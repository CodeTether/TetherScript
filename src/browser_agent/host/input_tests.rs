use crate::browser_agent::BrowserPage;

#[test]
fn focus_press_and_blur_share_host_state() {
    let mut state = super::state::HostState::new();
    state.page = BrowserPage::from_html("mem://input", "<input id='entry'>");
    state.page.eval_js("let n=document.querySelector('#entry');n.addEventListener('focus',function(){this.setAttribute('data-focus','focused');});n.addEventListener('blur',function(){this.setAttribute('data-focus','blurred');});").unwrap();
    let selector = super::value::map(vec![("selector", super::value::string("#entry"))]);
    let key = super::value::map(vec![("key", super::value::string("A"))]);

    super::focus::invoke(&mut state, "focus", &selector).unwrap();
    assert_eq!(attribute(&mut state), "focused");
    super::keyboard::invoke(&mut state, &key).unwrap();
    let value = state
        .page
        .eval_js("document.querySelector('#entry').value")
        .unwrap();
    assert_eq!(value.display(), "A");
    super::focus::invoke(&mut state, "blur", &selector).unwrap();
    assert_eq!(attribute(&mut state), "blurred");
    assert!(super::keyboard::invoke(&mut state, &key).is_err());
}

fn attribute(state: &mut super::state::HostState) -> String {
    state
        .page
        .eval_js("document.querySelector('#entry').getAttribute('data-focus')")
        .unwrap()
        .display()
}
