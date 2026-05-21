use super::case::{assert_case, Case};
use tetherscript::browser_agent::page::diagnostics::VisualElementEvidence;
use tetherscript::browser_agent::BrowserPage;

const CASE: Case = Case {
    area: "css/css2",
    wpt_shape: "flex, grid/table evidence, and stacking context styles are visible in production diagnostics",
    unsupported: &["complete visual formatting model"],
};

pub fn run() {
    assert_case(&CASE);
    flex_layout_evidence();
    grid_layout_computed_style_evidence();
    table_colspan_rowspan_sizing_evidence();
    transform_and_opacity_stacking_context_isolation();
}

fn flex_layout_evidence() {
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

fn grid_layout_computed_style_evidence() {
    let html = "<style>#grid{display:grid;grid-template-columns:40px 60px;gap:2px}\
        #cell{width:40px;height:10px}</style><div id='grid'><div id='cell'>G</div></div>";
    let page = BrowserPage::from_html("mem://grid", html);
    let report = page.production_debug_report();
    let grid = find(&report.visual_elements, "#grid");

    assert_eq!(grid.computed_styles.get("display").unwrap(), "grid");
    assert_eq!(
        grid.computed_styles.get("grid-template-columns").unwrap(),
        "40px 60px"
    );
}

fn table_colspan_rowspan_sizing_evidence() {
    let html = "<table id='tbl' style='width:120px'><tr><td id='wide' colspan='2' style='width:80px;height:10px'>A</td></tr>\
        <tr><td id='tall' rowspan='2' style='width:40px;height:20px'>B</td><td id='cell' style='width:40px;height:10px'>C</td></tr></table>";
    let page = BrowserPage::from_html("mem://table", html);
    let report = page.production_debug_report();
    let table = find(&report.visual_elements, "#tbl");
    let wide = find(&report.visual_elements, "#wide");
    let tall = find(&report.visual_elements, "#tall");

    assert!(table.visible);
    assert!(
        wide.bounds.width >= 80,
        "colspan cell should keep declared width evidence"
    );
    assert!(
        tall.bounds.height >= 20,
        "rowspan cell should keep declared height evidence"
    );
}

fn transform_and_opacity_stacking_context_isolation() {
    let html = "<div id='base' style='position:absolute;left:0;top:0;width:20px;height:20px;z-index:1;background:red'></div>\
        <div id='layer' style='position:absolute;left:0;top:0;width:20px;height:20px;z-index:2;opacity:0.5;transform:translateX(4px);background:blue'></div>";
    let page = BrowserPage::from_html("mem://stacking", html);
    let report = page.production_debug_report();
    let layer = find(&report.visual_elements, "#layer");

    assert_eq!(layer.computed_styles.get("opacity").unwrap(), "0.5");
    assert_eq!(
        layer.computed_styles.get("transform").unwrap(),
        "translateX(4px)"
    );
    assert_eq!(layer.computed_styles.get("z-index").unwrap(), "2");
}

fn find<'a>(items: &'a [VisualElementEvidence], selector: &str) -> &'a VisualElementEvidence {
    items
        .iter()
        .find(|item| item.selector_candidates.contains(&selector.to_string()))
        .unwrap_or_else(|| panic!("missing visual element: {selector}"))
}
