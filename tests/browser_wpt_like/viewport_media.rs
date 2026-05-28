use super::case::{assert_case, Case};
use tetherscript::browser_agent::{
    BrowserPage, ColorScheme, ForcedColors, MediaEmulation, ReducedMotion,
};

const CASE: Case = Case {
    area: "css/mediaqueries",
    wpt_shape: "viewport resize and media emulation expose matchMedia objects and change listeners",
    unsupported: &["continuous prefers-color-scheme transitions"],
};

pub fn run() {
    assert_case(&CASE);
    viewport_and_media_state();
    match_media_returns_media_query_list_shape();
    match_media_change_listeners_and_removal();
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

fn match_media_returns_media_query_list_shape() {
    let mut page = BrowserPage::from_html("mem://match-media-shape", "<main>V</main>");
    let value = page
        .eval_js("let m=window.matchMedia('(min-width: 600px)'); typeof m + ':' + m.media + ':' + m.matches + ':' + typeof m.addEventListener + ':' + typeof m.removeEventListener")
        .unwrap();

    assert_eq!(
        value.display(),
        "object:(min-width: 600px):false:function:function"
    );
}

fn match_media_change_listeners_and_removal() {
    let mut page = BrowserPage::from_html("mem://match-media-change", "<main>V</main>");
    let value = page
        .eval_js("let m=matchMedia('(min-width: 600px)');let out='';let keep=function(e){out+='K'+e.matches+';';};let gone=function(){out+='G';};m.addEventListener('change',keep);m.addEventListener('change',gone);m.removeEventListener('change',gone);m.dispatchEvent({type:'change'});out")
        .unwrap();

    assert_eq!(value.display(), "Kfalse;");
}
