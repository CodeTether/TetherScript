use crate::browser_agent::BrowserPage;

#[test]
fn clear_removes_shared_and_per_tab_storage() {
    let mut state = super::super::super::state::HostState::new();
    state.page.goto_html("https://app.test/", "");
    state
        .page
        .session
        .session_storage
        .entry("https://app.test".into())
        .or_default()
        .insert("tab".into(), "one".into());
    state.page.indexed_db_put("app", "s", "k", "v").unwrap();
    state
        .context_state
        .borrow_mut()
        .cookies
        .push(crate::browser_session::Cookie {
            name: "a".into(),
            value: "b".into(),
            domain: "app.test".into(),
            path: "/".into(),
            secure: false,
            http_only: false,
            same_site: crate::browser_cookie::SameSite::Lax,
            expires_at: None,
            host_only: true,
        });
    let mut tab = BrowserPage::new(Default::default());
    tab.attach_context_state(state.context_state.clone(), false);
    tab.session
        .session_storage
        .entry("https://app.test".into())
        .or_default()
        .insert("tab".into(), "two".into());
    state.tabs.push(Some(tab));
    assert!(super::clear::invoke(&mut state).truthy());
    assert!(state.context_state.borrow().is_empty());
    assert!(state.page.session.session_storage.is_empty());
    assert!(state.tabs[1]
        .as_ref()
        .unwrap()
        .session
        .session_storage
        .is_empty());
}
