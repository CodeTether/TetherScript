use super::case::{assert_case, Case};
use tetherscript::browser_agent::{
    BrowserPage, ColorScheme, ForcedColors, MediaEmulation, ReducedMotion,
};

#[path = "viewport_media_match.rs"]
mod match_media;

const CASE: Case = Case {
    area: "css/mediaqueries",
    wpt_shape: "viewport resize and scroll synchronize VisualViewport metrics and events",
    unsupported: &["pinch zoom and visualViewport scrollend events"],
};

pub fn run() {
    assert_case(&CASE);
    viewport_and_media_state();
    visual_viewport_metrics_and_events();
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

fn visual_viewport_metrics_and_events() {
    let mut page = BrowserPage::from_html("mem://visual-viewport", "<main>V</main>");
    page.eval_js("let v=visualViewport;let seen='';v.addEventListener('resize',function(){seen+='R'+v.width+'x'+v.height+';';});v.addEventListener('scroll',function(){seen+='S'+v.pageLeft+','+v.pageTop+';';});")
        .unwrap();
    page.set_viewport_size(120, 40).unwrap();
    let value = page
        .eval_js("scrollTo(3,5);[v.width,v.height,v.pageLeft,v.pageTop,seen].join('|')")
        .unwrap();

    assert_eq!(value.display(), "120|40|3|5|R120x40;S3,5;");
}
