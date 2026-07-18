use crate::browser_agent::BrowserPage;

#[test]
fn type_honors_cancellation_and_focuses_for_empty_text() {
    let mut state = super::super::super::state::HostState::new();
    state.page = BrowserPage::from_html("mem://cancel", "<input id='entry' value='a'>");
    state.page.eval_js("let n=document.getElementById('entry');n.addEventListener('keydown',function(e){e.preventDefault();});n.addEventListener('keyup',function(e){this.setAttribute('data-up',e.key);});").unwrap();
    super::super::invoke(&mut state, "type", &super::payload("x")).unwrap();
    assert_eq!(
        super::eval(&mut state, "document.getElementById('entry').value"),
        "a"
    );
    assert_eq!(
        super::eval(
            &mut state,
            "document.getElementById('entry').getAttribute('data-up')"
        ),
        "x"
    );

    let mut empty = super::super::super::state::HostState::new();
    empty.page = BrowserPage::from_html("mem://empty", "<input id='entry'>");
    empty.page.eval_js("document.getElementById('entry').addEventListener('focus',function(){this.setAttribute('data-focused','yes');});").unwrap();
    super::super::invoke(&mut empty, "type", &super::payload("")).unwrap();
    assert_eq!(
        super::eval(
            &mut empty,
            "document.getElementById('entry').getAttribute('data-focused')"
        ),
        "yes"
    );
}
