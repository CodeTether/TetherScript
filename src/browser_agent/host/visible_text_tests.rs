use crate::browser_agent::BrowserPage;

#[test]
fn visible_text_excludes_non_rendered_subtrees() {
    let page = BrowserPage::from_html(
        "mem://visible-text",
        "<main>Shown <span hidden>hidden</span><b>text</b>\
         <p style='display:none'>gone</p><i style='visibility:hidden'>invisible</i>\
         <script>secret()</script><style>main{color:red}</style></main>",
    );

    assert_eq!(super::visible_text::value(&page), "Shown text");
}
