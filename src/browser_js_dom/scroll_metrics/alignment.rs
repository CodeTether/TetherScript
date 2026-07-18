use super::*;

#[derive(Clone, Copy)]
pub(super) enum Alignment {
    Start,
    Center,
    End,
    Nearest,
}

pub(super) fn options(value: Option<&JsValue>) -> (Alignment, Alignment) {
    match value {
        Some(JsValue::Bool(false)) => (Alignment::End, Alignment::Nearest),
        Some(JsValue::Object(object)) => {
            let object = object.borrow();
            (
                parse(object.get("block"), Alignment::Start),
                parse(object.get("inline"), Alignment::Nearest),
            )
        }
        _ => (Alignment::Start, Alignment::Nearest),
    }
}

pub(super) fn axis(
    start: i64,
    size: i64,
    viewport_start: i64,
    viewport_size: i64,
    alignment: Alignment,
) -> i64 {
    match alignment {
        Alignment::Start => start,
        Alignment::Center => start.saturating_sub((viewport_size - size) / 2),
        Alignment::End => start.saturating_add(size).saturating_sub(viewport_size),
        Alignment::Nearest if start < viewport_start => start,
        Alignment::Nearest if start.saturating_add(size) > viewport_start + viewport_size => {
            start.saturating_add(size).saturating_sub(viewport_size)
        }
        Alignment::Nearest => viewport_start,
    }
}

fn parse(value: Option<&JsValue>, default: Alignment) -> Alignment {
    match value.map(JsValue::display).unwrap_or_default().as_str() {
        "start" => Alignment::Start,
        "center" => Alignment::Center,
        "end" => Alignment::End,
        "nearest" => Alignment::Nearest,
        _ => default,
    }
}
