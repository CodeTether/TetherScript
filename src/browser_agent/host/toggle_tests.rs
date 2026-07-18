use crate::browser_agent::BrowserPage;

#[test]
fn toggle_inverts_checkbox_state_and_dispatches_events() {
    let mut state = super::super::super::state::HostState::new();
    state.page = BrowserPage::from_html(
        "mem://toggle",
        "<input id='ok' type='checkbox'><script>let log='';let n=document.getElementById('ok');n.addEventListener('click',function(){log=log+'c'});n.addEventListener('input',function(){log=log+'i'});n.addEventListener('change',function(){log=log+'h'});</script>",
    );
    state.page.run_scripts().unwrap();
    let payload = super::super::super::value::map(vec![(
        "selector",
        super::super::super::value::string("#ok"),
    )]);

    super::invoke(&mut state, &payload).unwrap();
    assert_eq!(
        eval(&mut state, "document.getElementById('ok').checked"),
        "true"
    );
    super::invoke(&mut state, &payload).unwrap();

    assert_eq!(
        eval(&mut state, "document.getElementById('ok').checked"),
        "false"
    );
    assert_eq!(eval(&mut state, "log"), "cihcih");
}

fn eval(state: &mut super::super::super::state::HostState, script: &str) -> String {
    state.page.eval_js(script).unwrap().display()
}
