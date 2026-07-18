use super::case::{assert_dom, Case, DomCase};

const CASE: DomCase = DomCase {
    case: Case {
        area: "selectors-api",
        wpt_shape: "compound, combinator, attribute, and structural pseudo selectors",
        unsupported: &[
            "selector lists, namespaces, and CSS escaping",
            "dynamic-state and relational pseudo-classes",
            "invalid selector taxonomy",
        ],
    },
    html: "<main><section id='group'><p>A</p></section>\
        <button id='save' class='primary action' data-state='ready'>Save</button>\
        <button id='cancel' class='secondary'>Cancel</button></main>",
    script: "let actions=document.querySelectorAll('button.action');\
        let ready=document.querySelector('[data-state=ready]');\
        let child=document.querySelector('main > section:first-child');\
        let adjacent=document.querySelector('section + button');\
        let sibling=document.querySelector('section ~ button:last-child');\
        let nth=document.querySelector('button:nth-child(3)');\
        let negated=document.querySelector('button:not(.secondary)');\
        [actions.length,actions[0].id,ready.textContent,child.id,adjacent.id,\
        sibling.id,nth.id,negated.id].join(':');",
    expect: "1:save:Save:group:save:cancel:cancel:save",
};

pub fn run() {
    assert_dom(&CASE);
}
