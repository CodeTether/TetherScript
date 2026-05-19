use super::case::{assert_dom, Case, DomCase};

const CASE: DomCase = DomCase {
    case: Case {
        area: "selectors-api",
        wpt_shape: "querySelector/querySelectorAll class and attribute matching",
        unsupported: &[
            "full selector grammar",
            "pseudo-classes",
            "invalid selector taxonomy",
        ],
    },
    html: "<main><button id='save' class='primary action' data-state='ready'>Save</button>\
        <button id='cancel' class='secondary'>Cancel</button></main>",
    script: "let actions=document.querySelectorAll('button.action');\
        let ready=document.querySelector('[data-state=ready]');\
        actions.length+':'+actions[0].id+':'+ready.textContent;",
    expect: "1:save:Save",
};

pub fn run() {
    assert_dom(&CASE);
}
