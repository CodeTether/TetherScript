use crate::browser_agent::BrowserPage;
use crate::value::Value;

#[test]
fn select_and_close_preserve_independent_pages() {
    let mut state = super::super::super::state::HostState::new();
    state.page = BrowserPage::from_html("mem://one", "<main>one</main>");
    state.tabs.push(Some(BrowserPage::from_html(
        "mem://two",
        "<main>two</main>",
    )));

    super::select::activate(&mut state, 1).unwrap();
    assert_eq!(state.page.session.url, "mem://two");
    super::close::close(&mut state, 0).unwrap();

    assert_eq!(state.tabs.len(), 1);
    assert_eq!(state.active_tab, 0);
    let Value::List(tabs) = super::value::list(&state) else {
        panic!("expected tabs list")
    };
    assert_eq!(tabs.borrow().len(), 1);
}
