use super::case::{assert_case, Case};
use tetherscript::browser_agent::page::diagnostics::VisualElementEvidence;
use tetherscript::browser_agent::BrowserPage;

const CASE: Case = Case {
    area: "css/css2",
    wpt_shape: "flex layout exposes adjacent visual evidence",
    unsupported: &["CSS grid", "complete visual formatting model"],
};

pub fn run() {
    assert_case(&CASE);
    let html = "<style>#row{display:flex;width:80px}\
        #a,#b{width:40px;height:10px}</style>\
        <div id='row'><div id='a'></div><div id='b'></div></div>";
    let page = BrowserPage::from_html("mem://layout", html);
    let report = page.production_debug_report();
    let a = find(&report.visual_elements, "#a");
    let b = find(&report.visual_elements, "#b");
    assert!(a.visible);
    assert!(b.visible);
    assert_eq!(b.bounds.x, a.bounds.x + 40);
}

fn find<'a>(items: &'a [VisualElementEvidence], selector: &str) -> &'a VisualElementEvidence {
    items
        .iter()
        .find(|item| item.selector_candidates.contains(&selector.to_string()))
        .unwrap_or_else(|| panic!("missing visual element: {selector}"))
}
