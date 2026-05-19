use super::case::{assert_dom, Case, DomCase};

const CASE: DomCase = DomCase {
    case: Case {
        area: "selectors-api",
        wpt_shape: "invalid selector currently resolves to no matches",
        unsupported: &["DOMException SyntaxError for invalid selectors"],
    },
    html: "<main><p>Text</p></main>",
    script: "let one=document.querySelector('[');\
        let all=document.querySelectorAll('[');(one===null)+':'+all.length;",
    expect: "true:0",
};

pub fn run() {
    assert_dom(&CASE);
}
