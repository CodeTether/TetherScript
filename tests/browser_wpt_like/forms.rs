use super::case::{assert_dom, Case, DomCase};

const CASE: DomCase = DomCase {
    case: Case {
        area: "html/semantics/forms",
        wpt_shape: "reset and requestSubmit(submitter) expose form defaults",
        unsupported: &[
            "constraint validation",
            "full form-associated custom elements",
        ],
    },
    html: "<form id='f'><input id='q' name='q' value='rust'>\
        <input id='ok' type='checkbox' checked>\
        <button id='go' name='commit' value='yes'>Go</button>\
        <button id='reset' type='reset'>Reset</button></form>",
    script: "let f=document.getElementById('f');let q=document.getElementById('q');\
        let ok=document.getElementById('ok');q.value='changed';ok.checked=false;\
        document.getElementById('reset').click();\
        let data=f.requestSubmit(document.getElementById('go')).data;\
        q.value+':'+ok.checked+':'+data.get('commit');",
    expect: "rust:true:yes",
};

pub fn run() {
    assert_dom(&CASE);
}
