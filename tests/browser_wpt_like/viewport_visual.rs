use tetherscript::browser_agent::BrowserPage;

pub(super) fn metrics_and_events() {
    let mut page = BrowserPage::from_html("mem://visual-viewport", "<main>V</main>");
    page.eval_js("let v=visualViewport;let seen='';v.addEventListener('resize',function(){seen+='R'+v.width+'x'+v.height+';';});v.addEventListener('scroll',function(){seen+='S'+v.pageLeft+','+v.pageTop+';';});v.addEventListener('scrollend',function(e){seen+='E'+e.isTrusted+';';});")
        .unwrap();
    page.set_viewport_size(120, 40).unwrap();
    let value = page
        .eval_js("scrollTo(3,5);[v.width,v.height,v.pageLeft,v.pageTop,seen].join('|')")
        .unwrap();

    assert_eq!(value.display(), "120|40|3|5|R120x40;S3,5;Etrue;");
}
