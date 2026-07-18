use super::*;

#[path = "scroll_state.rs"]
mod state;
#[path = "scroll_write.rs"]
mod write;

#[derive(Clone, Copy)]
pub(in crate::browser_js) struct Metrics {
    pub x: i64,
    pub y: i64,
    pub width: i64,
    pub height: i64,
}

pub(super) fn register(window: &JsValue) {
    state::register(window);
}

pub(in crate::browser_js) fn metrics() -> Metrics {
    let Some(window) = state::current() else {
        return Metrics {
            x: 0,
            y: 0,
            width: 80,
            height: 24,
        };
    };
    let window = window.borrow();
    Metrics {
        x: number(window.get("scrollX")),
        y: number(window.get("scrollY")),
        width: number(window.get("innerWidth")).max(1),
        height: number(window.get("innerHeight")).max(1),
    }
}

pub(in crate::browser_js) fn to(x: i64, y: i64) -> Result<bool, String> {
    write::to(x, y)
}

fn number(value: Option<&JsValue>) -> i64 {
    value
        .and_then(|value| value.display().parse::<f64>().ok())
        .unwrap_or(0.0)
        .trunc() as i64
}
