use super::case::{assert_case, Case};
use tetherscript::browser_agent::{BrowserPage, FilePayload, Locator};

const CASE: Case = Case {
    area: "html/semantics/forms/file-upload-download",
    wpt_shape: "file input upload metadata and anchor download records are observable",
    unsupported: &["real filesystem file picker", "streamed download bodies"],
};

pub fn run() {
    assert_case(&CASE);
    let html = "<input id='u' type='file'>\
        <a id='dl' href='https://example.test/file.bin' download='out.bin'>Save</a>";
    let mut page = BrowserPage::from_html("https://example.test", html);
    let files = vec![FilePayload::new(
        "note.txt",
        "text/plain",
        b"hello".to_vec(),
    )];
    page.set_input_files(&Locator::css("#u"), files).unwrap();
    page.click(&Locator::css("#dl")).unwrap();
    assert!(page.session.html.contains("data-agent-file-count=\"1\""));
    assert_eq!(page.downloads().len(), 1);
    assert_eq!(page.downloads()[0].suggested_filename, "out.bin");
}
