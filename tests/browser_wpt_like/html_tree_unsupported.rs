use super::case::{assert_case, Case};
use tetherscript::browser::{parse_html, query_selector, text_content};

const CASE: Case = Case {
    area: "html/syntax/parsing",
    wpt_shape: "table rows parse without HTML5 implicit tbody insertion",
    unsupported: &["implicit table section insertion"],
};

pub fn run() {
    assert_case(&CASE);
    let doc = parse_html("<table><tr><td>A</td></tr></table>");
    let cells = query_selector(&doc, "td");
    assert_eq!(query_selector(&doc, "tbody").len(), 0);
    assert_eq!(cells.len(), 1);
    assert_eq!(text_content(&cells[0]), "A");
}
