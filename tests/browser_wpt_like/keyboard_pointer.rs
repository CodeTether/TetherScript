use super::case::{assert_case, Case};
use tetherscript::browser_agent::{BrowserPage, Locator};
use tetherscript::js::JsValue;

const CASE: Case = Case {
    area: "uievents",
    wpt_shape:
        "keyboard, composition, touch, and pointer device metadata mutate page-observable state",
    unsupported: &["real hardware IME and touch-device negotiation"],
};

pub fn run() {
    assert_case(&CASE);
    keyboard_text_and_pointer_hover();
    composition_events_sequence();
    touch_event_sequence_with_coordinate_metadata();
    pointer_event_pointer_type_differentiation();
}

fn keyboard_text_and_pointer_hover() {
    let html = "<input id='q' value='a'><button id='go'>Go</button>\
        <script>window.hoverLog='';let b=document.getElementById('go');\
        b.addEventListener('mouseover',function(){window.hoverLog+='over>';});\
        b.addEventListener('mouseenter',function(){window.hoverLog+='enter>';});\
        b.addEventListener('mousemove',function(){window.hoverLog+='move';});</script>";
    let mut page = BrowserPage::from_html("mem://input", html);
    page.run_scripts().unwrap();
    page.press(&Locator::css("#q"), 'b').unwrap();
    page.hover(&Locator::css("#go")).unwrap();
    assert_eq!(
        page.eval_js("document.getElementById('q').value+':'+window.hoverLog")
            .unwrap(),
        JsValue::String("ab:over>enter>move".into())
    );
}

fn composition_events_sequence() {
    let html = "<input id='ime'><script>let q=document.getElementById('ime');window.log='';\
        for(let n of ['compositionstart','compositionupdate','compositionend']){q.addEventListener(n,function(e){window.log+=e.type+':'+e.data+'>';});}</script>";
    let mut page = BrowserPage::from_html("mem://composition", html);
    page.run_scripts().unwrap();

    let value = page
        .eval_js("let q=document.getElementById('ime');q.dispatchEvent(new CompositionEvent('compositionstart',{data:''}));q.dispatchEvent(new CompositionEvent('compositionupdate',{data:'あ'}));q.dispatchEvent(new CompositionEvent('compositionend',{data:'あ'}));window.log")
        .unwrap();

    assert_eq!(
        value.display(),
        "compositionstart:>compositionupdate:あ>compositionend:あ>"
    );
}

fn touch_event_sequence_with_coordinate_metadata() {
    let html = "<button id='tap' style='width:20px;height:10px'>Tap</button><script>let b=document.getElementById('tap');window.log='';\
        b.addEventListener('touchstart',function(e){window.log+='start:'+e.touches.length+':'+e.changedTouches[0].clientX+'>';});\
        b.addEventListener('touchmove',function(e){window.log+='move:'+e.touches.length+':'+e.changedTouches[0].clientY+'>';});\
        b.addEventListener('touchend',function(e){window.log+='end:'+e.touches.length+':'+e.changedTouches.length;});</script>";
    let mut page = BrowserPage::from_html("mem://touch", html);
    page.run_scripts().unwrap();

    page.touch_sequence(&Locator::css("#tap")).unwrap();

    let value = page.eval_js("window.log").unwrap().display();
    assert!(
        value.starts_with("start:1:"),
        "unexpected touch log: {value}"
    );
    assert!(value.contains(">move:1:"), "unexpected touch log: {value}");
    assert!(value.ends_with(">end:0:1"), "unexpected touch log: {value}");
}

fn pointer_event_pointer_type_differentiation() {
    let mut page = BrowserPage::from_html("mem://pointer-types", "<main></main>");
    let value = page
        .eval_js("let m=new PointerEvent('pointerdown',{pointerType:'mouse'});let p=new PointerEvent('pointerdown',{pointerType:'pen'});let t=new PointerEvent('pointerdown',{pointerType:'touch'});m.pointerType+':'+p.pointerType+':'+t.pointerType")
        .unwrap();

    assert_eq!(value.display(), "mouse:pen:touch");
}
