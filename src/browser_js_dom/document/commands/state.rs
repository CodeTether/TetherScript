use super::*;

thread_local! {
    static CLIPBOARD_TEXT: RefCell<Option<String>> = const { RefCell::new(None) };
}

pub(super) fn reset() {
    CLIPBOARD_TEXT.with(|text| *text.borrow_mut() = None);
}

pub(super) fn available() -> bool {
    CLIPBOARD_TEXT.with(|text| text.borrow().is_some())
}

pub(super) fn read() -> Option<String> {
    CLIPBOARD_TEXT.with(|text| text.borrow().clone())
}

pub(super) fn write(text: String) {
    CLIPBOARD_TEXT.with(|target| *target.borrow_mut() = Some(text));
}
