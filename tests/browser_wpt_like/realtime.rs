use super::case::{assert_case, Case};
use tetherscript::browser_agent::{BrowserPage, RealtimeKind};
use tetherscript::js::JsValue;

const CASE: Case = Case {
    area: "websockets/eventsource",
    wpt_shape: "WebSocket send and EventSource/WebSocket message events are observable",
    unsupported: &[
        "real socket transport",
        "binary frames and EventSource reconnection timing",
    ],
};

pub fn run() {
    assert_case(&CASE);
    let html = "<p id='ws'></p><p id='es'></p>";
    let script = "let ws=WebSocket('ws://agent');\
        ws.onopen=function(){ws.send('ping');};\
        ws.onmessage=function(e){document.getElementById('ws').textContent=e.data;};\
        let es=EventSource('/events');\
        es.onmessage=function(e){document.getElementById('es').textContent=e.data;};";
    let mut page = BrowserPage::from_html("https://app.test", html);
    page.eval_js(script).unwrap();
    let connections = page.realtime_connections().unwrap();
    let ws = connections
        .iter()
        .find(|conn| conn.kind == RealtimeKind::WebSocket)
        .unwrap();
    let es = connections
        .iter()
        .find(|conn| conn.kind == RealtimeKind::EventSource)
        .unwrap();
    page.inject_websocket_message(ws.id, "pong").unwrap();
    page.inject_event_source_message(es.id, "stream").unwrap();
    assert_eq!(page.realtime_outbound_messages().unwrap()[0].data, "ping");
    assert_eq!(
        page.eval_js("document.getElementById('ws').textContent+'|'+document.getElementById('es').textContent")
            .unwrap(),
        JsValue::String("pong|stream".into())
    );
}
