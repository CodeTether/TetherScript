use crate::browser_agent::{resolve, BrowserPage, Locator};

#[test]
fn coordinate_click_targets_topmost_element_with_requested_coordinates() {
    let html = "<button id='go'>Go</button><script>let n=document.getElementById('go');n.addEventListener('click',function(e){this.setAttribute('data-hit',e.isTrusted+':'+e.clientX+','+e.clientY);});</script>";
    let mut page = BrowserPage::from_html("mem://point", html);
    page.run_scripts().unwrap();
    let bounds = resolve::resolve(&page.session, page.viewport_width, &Locator::css("#go"))
        .unwrap()
        .bounds;
    let x = bounds.x + bounds.width / 2;
    let y = bounds.y + bounds.height / 2;
    let target = crate::browser_agent::hit::target_at(&page.session, page.viewport_width, x, y)
        .unwrap_or_else(|| panic!("no target in {bounds:?} at {x},{y}"));
    assert_eq!(target.label, "button#go");

    let report = page.click_at(x, y).unwrap();

    assert_eq!(report.action, "mouse_click");
    assert_eq!(
        page.eval_js("document.getElementById('go').getAttribute('data-hit')")
            .unwrap()
            .display(),
        format!("true:{x},{y}")
    );
}
