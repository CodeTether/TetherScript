use super::case::{assert_case, Case};
use tetherscript::browser_agent::{
    BrowserPage, ColorScheme, ForcedColors, MediaEmulation, ReducedMotion,
};

#[path = "viewport_media_match.rs"]
mod match_media;
#[path = "viewport_orientation.rs"]
mod orientation;
#[path = "viewport_visual.rs"]
mod visual;

const CASE: Case = Case {
    area: "css/mediaqueries, css/cssom-view, screen-orientation",
    wpt_shape: "viewport metrics, events, media queries, and screen orientation locks",
    unsupported: &[
        "pinch zoom and visualViewport scrollend events",
        "device sensor-driven orientation changes",
    ],
};

pub fn run() {
    assert_case(&CASE);
    viewport_and_media_state();
    visual::metrics_and_events();
    orientation::locks_and_unlocks();
    orientation::exposes_legacy_window_orientation();
    match_media::object_shape();
    match_media::listeners_and_removal();
}

fn viewport_and_media_state() {
    let mut page = BrowserPage::from_html("mem://viewport", "<main>V</main>");
    page.set_viewport_size(320, 480).unwrap();
    assert_eq!(page.viewport().width, 320);
    assert_eq!(page.viewport().height, 480);
    page.set_color_scheme(ColorScheme::Dark);
    page.set_reduced_motion(ReducedMotion::Reduce);
    page.set_forced_colors(ForcedColors::Active);
    let media = page.media();
    assert_eq!(
        media,
        MediaEmulation {
            color_scheme: ColorScheme::Dark,
            reduced_motion: ReducedMotion::Reduce,
            forced_colors: ForcedColors::Active,
        }
    );
}
