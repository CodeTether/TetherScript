use super::{BrowserPage, Locator};

#[test]
fn wait_for_visible_settles_timer_created_dom() {
    let html = "<main id='app'></main><script>queueMicrotask(function(){ let b=document.createElement('button'); b.setAttribute('id','later'); b.textContent='Go'; document.getElementById('app').appendChild(b); });</script>";
    let mut page = BrowserPage::from_html("mem://wait", html);

    let bounds = page.wait_for_visible(&Locator::css("#later")).unwrap();

    assert!(bounds.visible());
    assert!(page.session.html.contains("id=\"later\""));
}

#[test]
fn click_retries_until_target_is_actionable() {
    let html = "<input id='go' type='checkbox' disabled><script>setTimeout(function(){ document.getElementById('go').removeAttribute('disabled'); }, 0);</script>";
    let mut page = BrowserPage::from_html("mem://click-wait", html);

    page.click(&Locator::css("#go")).unwrap();

    assert!(page.session.html.contains("checked=\"\""));
}

#[test]
fn click_waits_for_two_matching_layout_observations() {
    let html = "<button id='move' style='position:absolute;left:0px'>Go</button>\
        <script style='display:none'>let b=document.getElementById('move');\
        b.addEventListener('click',function(){window.clicked='yes';});\
        setTimeout(function(){b.setAttribute('style','position:absolute;left:12px');},0);</script>";
    let mut page = BrowserPage::from_html("mem://stable-click", html);

    let report = page.click(&Locator::css("#move")).unwrap();

    assert_eq!(report.bounds.x, 12);
    assert_eq!(page.eval_js("window.clicked").unwrap().display(), "yes");
}

#[test]
fn timeout_reports_last_locator_error() {
    let mut page = BrowserPage::from_html("mem://missing", "<main></main>");
    page.set_default_timeout_ticks(1);

    let err = page.click(&Locator::css("#missing")).unwrap_err();

    assert!(err.contains("click timed out after 1 ticks"));
    assert!(err.contains("locator css(\"#missing\")"));
    assert!(err.contains("matched no elements"));
}
