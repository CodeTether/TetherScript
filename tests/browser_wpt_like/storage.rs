use super::case::{assert_dom, Case, DomCase};

const CASE: DomCase = DomCase {
    case: Case {
        area: "webstorage",
        wpt_shape: "Storage set/get/remove/clear update key order and length",
        unsupported: &["quota errors", "cross-document storage events"],
    },
    html: "<main></main>",
    script: "localStorage.setItem('a',1);localStorage.setItem('b','two');\
        localStorage.setItem('a','one');\
        let before=localStorage.length+':'+localStorage.key(0)+':'\
        +localStorage.getItem('a')+':'+localStorage.getItem('missing');\
        localStorage.removeItem('b');let after=localStorage.length+':'\
        +localStorage.key(1);localStorage.clear();\
        before+'|'+after+'|'+localStorage.length+':'+localStorage.getItem('a');",
    expect: "2:a:one:null|1:null|0:null",
};

pub fn run() {
    assert_dom(&CASE);
}
