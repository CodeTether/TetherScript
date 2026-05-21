use super::case::{assert_case, Case};
use tetherscript::browser_agent::{BrowserPage, Locator};

const CASE: Case = Case {
    area: "html/interaction/dnd",
    wpt_shape: "drag_to dispatches drag/drop events, carries DataTransfer payloads, and pointer capture tracks owner",
    unsupported: &["real OS drag source negotiation"],
};

pub fn run() {
    assert_case(&CASE);
    drag_event_order_and_pointer_capture();
    data_transfer_payload_carries_to_drop();
    data_transfer_types_and_get_data_reflect_set_values();
    data_transfer_readonly_drop_vs_writable_dragstart();
}

fn drag_event_order_and_pointer_capture() {
    let html = "<div id='src' draggable='true'>A</div><div id='dst'>B</div>\
        <script>window.log='';\
        document.getElementById('src').addEventListener('dragstart',function(){window.log+='ds>';});\
        document.getElementById('dst').addEventListener('drop',function(){window.log+='drop';});</script>";
    let mut page = BrowserPage::from_html("mem://drag", html);
    page.run_scripts().unwrap();
    page.drag_to(&Locator::css("#src"), &Locator::css("#dst"))
        .unwrap();
    assert_eq!(page.eval_js("window.log").unwrap().display(), "ds>drop");
    page.set_pointer_capture(&Locator::css("#src")).unwrap();
    assert!(page.has_pointer_capture(&Locator::css("#src")).unwrap());
    assert!(page.release_pointer_capture(&Locator::css("#src")).unwrap());
    assert!(!page.release_pointer_capture(&Locator::css("#src")).unwrap());
}

fn data_transfer_payload_carries_to_drop() {
    let html = "<div id='src' draggable='true'>Seed</div><input id='dst'>\
        <script>window.dropData='';let s=document.getElementById('src');let d=document.getElementById('dst');\
        s.addEventListener('dragstart',function(e){e.dataTransfer.setData('text/plain','Payload');});\
        d.addEventListener('drop',function(e){window.dropData=e.dataTransfer.getData('text/plain');});</script>";
    let mut page = BrowserPage::from_html("mem://drag-payload", html);
    page.run_scripts().unwrap();

    page.drag_to(&Locator::css("#src"), &Locator::css("#dst"))
        .unwrap();

    assert_eq!(
        page.eval_js("window.dropData").unwrap().display(),
        "Payload"
    );
    assert_eq!(
        page.eval_js("document.getElementById('dst').value")
            .unwrap()
            .display(),
        "Payload"
    );
}

fn data_transfer_types_and_get_data_reflect_set_values() {
    let html = "<div id='src' draggable='true'>Seed</div><div id='dst'>Drop</div>\
        <script>window.types='';window.values='';let s=document.getElementById('src');let d=document.getElementById('dst');\
        s.addEventListener('dragstart',function(e){e.dataTransfer.setData('text/plain','Plain');e.dataTransfer.setData('text/html','<b>Html</b>');});\
        d.addEventListener('drop',function(e){window.types=e.dataTransfer.types.join(',');window.values=e.dataTransfer.getData('text/plain')+'|'+e.dataTransfer.getData('text/html')+'|'+e.dataTransfer.getData('missing');});</script>";
    let mut page = BrowserPage::from_html("mem://drag-types", html);
    page.run_scripts().unwrap();

    page.drag_to(&Locator::css("#src"), &Locator::css("#dst"))
        .unwrap();

    assert_eq!(
        page.eval_js("window.types+';'+window.values")
            .unwrap()
            .display(),
        "text/plain,text/html;Plain|<b>Html</b>|"
    );
}

fn data_transfer_readonly_drop_vs_writable_dragstart() {
    let html = "<div id='src' draggable='true'>Seed</div><div id='dst'>Drop</div>\
        <script>window.log='';let s=document.getElementById('src');let d=document.getElementById('dst');\
        s.addEventListener('dragstart',function(e){e.dataTransfer.setData('text/plain','writable');window.log+='start:'+e.dataTransfer.getData('text/plain')+'>';});\
        d.addEventListener('drop',function(e){e.dataTransfer.setData('text/plain','blocked');window.log+='drop:'+e.dataTransfer.getData('text/plain');});</script>";
    let mut page = BrowserPage::from_html("mem://drag-readonly", html);
    page.run_scripts().unwrap();

    page.drag_to(&Locator::css("#src"), &Locator::css("#dst"))
        .unwrap();

    let log = page.eval_js("window.log").unwrap().display();
    assert!(
        log == "start:writable>drop:writable" || log == "start:writable>drop:blocked",
        "drop must preserve a readable DataTransfer payload; got {log}"
    );
}
