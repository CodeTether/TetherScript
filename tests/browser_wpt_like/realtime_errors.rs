use super::case::{assert_case, Case};
use tetherscript::browser_agent::{BrowserPage, RealtimeEventKind};

const CASE: Case = Case {
    area: "websockets",
    wpt_shape: "failed WebSocket dispatches error/close metadata",
    unsupported: &[
        "network close code taxonomy",
        "browser socket handshake states",
    ],
};

pub fn run() {
    assert_case(&CASE);
    let mut page = BrowserPage::from_html("mem://rt", "<p id='out'></p>");
    page.eval_js(
        "let ws=WebSocket('ws://agent');ws.onerror=function(e){\
        document.getElementById('out').textContent=e.reason;};",
    )
    .unwrap();
    let id = page.realtime_connections().unwrap()[0].id;
    page.fail_websocket_connection(id, "refused").unwrap();
    let events = page.realtime_events().unwrap();
    assert!(page.session.html.contains(">refused</p>"));
    assert_eq!(events.last().unwrap().event, RealtimeEventKind::Close);
    assert_eq!(events[2].reason.as_deref(), Some("refused"));
}
