use tetherscript::browser_agent::{BrowserPage, ColorScheme, ForcedColors, Locator, ReducedMotion};

#[test]
fn media_queries_follow_viewport_and_emulated_preferences() {
    let html = "<style>#box{color:black;background:white;transition-duration:1s}\
        @media (min-width:100px) and (prefers-color-scheme:dark){\
        #box{color:purple;background:black}}\
        @media (prefers-reduced-motion:reduce){#box{transition-duration:0s}}\
        @media (forced-colors:active){#box{outline-color:CanvasText}}\
        </style><main id='box'></main>";
    let mut page = BrowserPage::from_html("mem://css-media", html);
    page.set_viewport_size(120, 80).unwrap();
    page.set_color_scheme(ColorScheme::Dark);
    page.set_reduced_motion(ReducedMotion::Reduce);
    page.set_forced_colors(ForcedColors::Active);
    let style = page.computed_style(&Locator::css("#box")).unwrap();

    assert_eq!(style.get("color"), Some("purple"));
    assert_eq!(style.get("background"), Some("black"));
    assert_eq!(style.get("transition-duration"), Some("0s"));
    assert_eq!(style.get("outline-color"), Some("CanvasText"));
}
