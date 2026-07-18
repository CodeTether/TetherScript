use crate::browser_agent::BrowserPage;

#[test]
fn exact_text_avoids_matching_ancestor_elements() {
    let mut state = super::super::super::state::HostState::new();
    state.page = BrowserPage::from_html(
        "mem://click-text",
        "<main><button id='target'>ready</button><p>other</p></main><script>document.querySelector('#target').addEventListener('click',function(){window.clicked=true;});</script>",
    );
    state.page.run_scripts().unwrap();
    let payload = super::super::super::value::map(vec![(
        "text",
        super::super::super::value::string("ready"),
    )]);
    super::invoke(&mut state, "click_text", &payload).unwrap();
    assert!(state.page.eval_js("window.clicked").unwrap().truthy());
}
