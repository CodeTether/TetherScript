use super::case::{assert_case, Case};
use tetherscript::browser_agent::{
    BrowserPage, ColorScheme, ForcedColors, MediaEmulation, ReducedMotion,
};

const CASE: Case = Case {
    area: "css/mediaqueries",
    wpt_shape: "viewport resize and media emulation change page-visible media state",
    unsupported: &[
        "matchMedia listener dispatch",
        "continuous prefers-color-scheme transitions",
    ],
};

pub fn run() {
    assert_case(&CASE);
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
