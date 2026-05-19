use tetherscript::browser_agent::page::diagnostics::VisualElementEvidence;
use tetherscript::browser_agent::BrowserPage;

fn report(page: &BrowserPage) -> Vec<VisualElementEvidence> {
    page.production_debug_report().visual_elements
}

fn find<'a>(elements: &'a [VisualElementEvidence], sel: &str) -> &'a VisualElementEvidence {
    elements
        .iter()
        .find(|e| e.selector_candidates.contains(&sel.to_string()))
        .unwrap_or_else(|| panic!("element not found: {sel}"))
}

#[test]
fn flex_position_absolute_and_z_index_interact() {
    let html = "<style>\
        .row{display:flex;width:80px}\
        .row>div{width:30px;height:10px}\
        #abs{position:absolute;top:50px;left:5px;width:20px;height:5px}\
        #over{position:absolute;top:0;left:0;width:20px;height:5px;z-index:2}\
    </style>\
    <div class='row'><div id='a'></div><div id='b'></div></div>\
    <div id='abs'></div>\
    <div id='over'></div>";
    let page = BrowserPage::from_html("mem://layout", html);
    let els = report(&page);
    let a = find(&els, "#a");
    let b = find(&els, "#b");
    let abs = find(&els, "#abs");
    let over = find(&els, "#over");
    assert!(a.visible);
    assert!(b.visible);
    assert_eq!(b.bounds.x, a.bounds.x + 30, "flex items adjacent");
    assert!(abs.visible);
    assert_eq!((abs.bounds.x, abs.bounds.y), (5, 50), "absolute position");
    assert!(over.visible);
    assert_eq!(over.computed_styles.get("z-index").unwrap(), "2");
}

#[test]
fn display_none_marks_invisible() {
    let html = "<style>#hidden{display:none;width:20px;height:10px}</style>\
        <div id='visible'>hi</div><div id='hidden'>no</div>";
    let page = BrowserPage::from_html("mem://hidden", html);
    let els = report(&page);
    assert!(find(&els, "#visible").visible);
    let hidden = find(&els, "#hidden");
    assert!(!hidden.visible);
    assert_eq!(hidden.computed_styles.get("display").unwrap(), "none");
}

#[test]
fn overflow_hidden_clips_visual_bounds() {
    let html = "<style>\
        .clip{overflow:hidden;width:10px;height:5px}\
        .child{width:20px;height:10px}\
    </style>\
    <div class='clip'><div id='child' class='child'></div></div>";
    let page = BrowserPage::from_html("mem://clip", html);
    let els = report(&page);
    let child = find(&els, "#child");
    assert!(child.visible);
    assert!(child.bounds.width <= 10, "clipped to container width");
    assert!(child.bounds.height <= 5, "clipped to container height");
}

#[test]
fn viewport_resize_changes_element_width() {
    let html = "<style>#full{width:100%;height:10px}</style><div id='full'></div>";
    let mut page = BrowserPage::from_html("mem://resize", html);
    page.viewport_width = 200;
    let wide = report(&page);
    assert_eq!(find(&wide, "#full").bounds.width, 200, "100% of 200px");
    page.viewport_width = 100;
    let narrow = report(&page);
    assert_eq!(find(&narrow, "#full").bounds.width, 100, "100% of 100px");
}

#[test]
fn zero_size_element_is_marked_invisible() {
    let html = "<style>#zero{width:0px;height:10px}</style><div id='zero'></div>";
    let page = BrowserPage::from_html("mem://z", html);
    let els = report(&page);
    let el = find(&els, "#zero");
    assert!(!el.visible);
    assert_eq!(el.bounds.width, 0);
}

#[test]
fn visibility_hidden_marks_invisible() {
    let html = "<style>#v{visibility:hidden;width:20px;height:10px}</style><div id='v'></div>";
    let page = BrowserPage::from_html("mem://vhidden", html);
    let els = report(&page);
    let el = find(&els, "#v");
    assert!(!el.visible);
    assert_eq!(el.computed_styles.get("visibility").unwrap(), "hidden");
}

#[test]
fn text_metrics_are_deterministic() {
    let html = "<style>#t{width:80px;height:20px}</style><div id='t'>ABCDE</div>";
    let page = BrowserPage::from_html("mem://text", html);
    let els = report(&page);
    let el = find(&els, "#t");
    assert!(el.visible);
    assert_eq!(el.text, "ABCDE");
    // The div is 80px wide per its CSS width
    assert_eq!(el.bounds.width, 80);
}

#[test]
fn inline_layout_wraps_text() {
    let html = "<style>#box{width:20px;height:20px}</style>\
        <div id='box'>Hello World</div>";
    let page = BrowserPage::from_html("mem://text", html);
    let els = report(&page);
    let el = find(&els, "#box");
    assert!(el.visible);
    assert!(el.text.contains("Hello"));
}
