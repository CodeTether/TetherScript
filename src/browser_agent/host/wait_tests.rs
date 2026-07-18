use std::time::{Duration, Instant};

use crate::browser_agent::BrowserPage;

#[path = "wait_tests_states.rs"]
mod states;

#[test]
fn wait_polls_scripts_and_honors_timeout_and_state() {
    let mut state = super::state::HostState::new();
    state.page = BrowserPage::from_html(
        "mem://wait",
        "<main id='app'></main><script>queueMicrotask(function(){let b=document.createElement('button');b.setAttribute('id','later');document.getElementById('app').appendChild(b);});</script>",
    );
    let later = payload(vec![
        ("selector", super::value::string("#later")),
        ("state", super::value::string("visible")),
        ("timeout_ms", crate::value::Value::Int(50)),
    ]);
    assert!(super::wait::invoke(&mut state, &later).unwrap().truthy());

    let missing = payload(vec![
        ("selector", super::value::string("#missing")),
        ("state", super::value::string("visible")),
        ("timeout_ms", crate::value::Value::Int(10)),
    ]);
    let started = Instant::now();
    let error = super::wait::invoke(&mut state, &missing).unwrap_err();
    assert!(started.elapsed() >= Duration::from_millis(10));
    assert!(error.contains("#missing") && error.contains("10ms"));
}

#[test]
fn network_idle_wait_runs_pending_page_work() {
    let mut state = super::state::HostState::new();
    state.page = BrowserPage::from_html("mem://idle", "<script>fetch('/settled');</script>");
    let idle = payload(vec![("network_idle", crate::value::Value::Bool(true))]);
    assert!(super::wait::invoke(&mut state, &idle).unwrap().truthy());
    assert_eq!(
        state.page.load_state(),
        crate::browser_agent::PageLoadState::NetworkIdle
    );
    assert!(state
        .page
        .network_events()
        .iter()
        .any(|event| event.url.ends_with("/settled")));
}

pub(super) fn payload(entries: Vec<(&str, crate::value::Value)>) -> crate::value::Value {
    super::value::map(entries)
}
