use super::*;

pub(super) fn install(window: &mut HashMap<String, JsValue>) {
    base64::install(window);
    typed_array::install(window);
    text::install(window);
    structured::install(window);
    crypto::install(window);
    dom_exception::install(window);
    dom_constructors::install(window);
    events::install(window);
    blob::install(window);
    clipboard_item::install(window);
    form_data::install(window);
    file_reader::install(window);
    notification::install(window);
    promise::install(window);
    url_pattern::install(window);
}
