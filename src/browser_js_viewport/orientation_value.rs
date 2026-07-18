#[derive(Clone, Copy)]
pub(super) struct Snapshot {
    pub kind: &'static str,
    pub angle: f64,
}

pub(super) fn viewport(width: f64, height: f64) -> Snapshot {
    if height > width {
        Snapshot {
            kind: "portrait-primary",
            angle: 90.0,
        }
    } else {
        primary("landscape-primary", 0.0)
    }
}

pub(super) fn requested(kind: &str, current: Snapshot) -> Result<Snapshot, String> {
    match kind {
        "any" => Ok(current),
        "natural" | "landscape" | "landscape-primary" => Ok(primary("landscape-primary", 0.0)),
        "landscape-secondary" => Ok(primary("landscape-secondary", 180.0)),
        "portrait" | "portrait-primary" => Ok(primary("portrait-primary", 90.0)),
        "portrait-secondary" => Ok(primary("portrait-secondary", 270.0)),
        _ => Err(format!(
            "NotSupportedError: screen.orientation.lock: invalid orientation '{kind}'"
        )),
    }
}

fn primary(kind: &'static str, angle: f64) -> Snapshot {
    Snapshot { kind, angle }
}
