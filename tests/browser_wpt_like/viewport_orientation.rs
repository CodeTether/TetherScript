use tetherscript::browser_agent::BrowserPage;

pub(super) fn locks_and_unlocks() {
    let mut page = BrowserPage::from_html("mem://screen-orientation", "<main>V</main>");
    page.set_viewport_size(20, 100).unwrap();
    let value = page
        .eval_js("let o=screen.orientation;let seen='';o.addEventListener('change',function(){window.orientationEventWidth=innerWidth;seen+=o.type+':'+o.angle+';';});let locked=o.lock('landscape-secondary');let invalid=o.lock('sideways');resizeTo(10,200);let during=o.type+':'+o.angle;o.unlock();[locked.__promise_state,invalid.__promise_state,during,o.type+':'+o.angle,window.orientationEventWidth,seen].join('|')")
        .unwrap();

    assert_eq!(
        value.display(),
        "fulfilled|rejected|landscape-secondary:180|portrait-primary:90|10|landscape-secondary:180;portrait-primary:90;"
    );
}
