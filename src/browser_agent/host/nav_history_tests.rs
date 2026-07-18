use crate::browser_agent::BrowserPage;

#[test]
fn traversal_restores_page_script_globals() {
    let mut state = super::super::super::state::HostState::new();
    state.page = BrowserPage::from_html("mem://one", "<script>window.pageName='one';</script>");
    state.page.run_scripts().unwrap();
    state
        .page
        .goto_html("mem://two", "<script>window.pageName='two';</script>");
    state.page.run_scripts().unwrap();
    super::navigate(&mut state, "back").unwrap();
    assert_eq!(
        state.page.eval_js("window.pageName").unwrap().display(),
        "one"
    );
    super::navigate(&mut state, "forward").unwrap();
    assert_eq!(
        state.page.eval_js("window.pageName").unwrap().display(),
        "two"
    );
}
