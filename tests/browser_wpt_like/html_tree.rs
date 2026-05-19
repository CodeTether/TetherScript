use super::case::{assert_case, Case};
use tetherscript::browser::{parse_html, query_selector, text_content};

const CASE: Case = Case {
    area: "html/syntax/parsing",
    wpt_shape: "basic tree construction preserves element order and entity text",
    unsupported: &[
        "HTML5 error recovery matrix",
        "doctype and namespace handling",
    ],
};

pub fn run() {
    assert_case(&CASE);
    let doc = parse_html("<main><p>A &amp; B</p><br><img src='x'><p>C</p></main>");
    let paragraphs = query_selector(&doc, "main p");
    let images = query_selector(&doc, "img[src='x']");
    assert_eq!(paragraphs.len(), 2);
    assert_eq!(images.len(), 1);
    assert_eq!(text_content(&paragraphs[0]), "A & B");
    assert_eq!(text_content(&paragraphs[1]), "C");
}
