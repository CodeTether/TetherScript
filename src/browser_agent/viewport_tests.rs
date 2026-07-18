use super::{BrowserPage, DeviceScale, Locator};

#[test]
fn set_viewport_updates_compat_fields() {
    let mut page = BrowserPage::from_html("mem://viewport", "<main>Viewport</main>");

    page.set_viewport_size(320, 640).unwrap();
    let viewport = page.viewport();

    assert_eq!(page.viewport_width, 320);
    assert_eq!(page.viewport_height, 640);
    assert_eq!(viewport.width, 320);
    assert_eq!(viewport.height, 640);
}

#[test]
fn set_viewport_updates_js_metrics_and_dispatches_resize() {
    let mut page = BrowserPage::from_html("mem://viewport-js", "<main>Viewport</main>");
    page.eval_js("let seen='';visualViewport.addEventListener('resize',function(e){seen=e.type+':'+e.isTrusted;});").unwrap();

    page.set_viewport_size(320, 640).unwrap();
    let value = page
        .eval_js(
            "[innerWidth,innerHeight,visualViewport.width,visualViewport.height,seen].join(':')",
        )
        .unwrap();

    assert_eq!(value.display(), "320:640:320:640:resize:true");
}

#[test]
fn clone_debug_and_eq_preserve_device_metadata() {
    let mut page = BrowserPage::from_html("mem://device", "<main>Device</main>");
    page.set_viewport_size(390, 844).unwrap();
    page.device_scale = DeviceScale::new(3.0, true).unwrap();

    let cloned = page.clone();

    assert_eq!(page, cloned);
    assert_eq!(cloned.viewport(), page.viewport());
    assert!(format!("{page:?}").contains("device_scale"));
}

#[test]
fn scroll_uses_configured_viewport_height() {
    let html = "<div style='height:40px'>Top</div><button id='go'>Go</button>";
    let mut page = BrowserPage::from_html("mem://scroll-viewport", html);
    page.set_viewport_size(80, 10).unwrap();

    page.click(&Locator::css("#go")).unwrap();

    assert!(page.session.scroll.y > 0);
}

#[test]
fn page_render_uses_viewport_and_device_scale() {
    let mut page = BrowserPage::from_html("mem://render", "<main>Hi</main>");
    page.set_viewport_size(12, 8).unwrap();
    page.device_scale = DeviceScale::new(2.0, false).unwrap();

    let image = page.render_raster().unwrap();

    assert_eq!(image.width, 24);
    assert_eq!(image.height, 16);
}
