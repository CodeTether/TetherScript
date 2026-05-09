//! JavaScript `document.cookie` projection state.

use std::cell::RefCell;

#[path = "browser_js_cookies/delete.rs"]
mod delete;
#[path = "browser_js_cookies/update.rs"]
mod update;

thread_local! {
    static COOKIE_JAR: RefCell<Vec<(String, String)>> = const { RefCell::new(Vec::new()) };
    static COOKIE_MUTATIONS: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
}

pub(crate) fn reset() {
    COOKIE_JAR.with(|cookies| cookies.borrow_mut().clear());
    COOKIE_MUTATIONS.with(|mutations| mutations.borrow_mut().clear());
}

pub(crate) fn seed(cookies: Vec<(String, String)>) {
    COOKIE_JAR.with(|jar| *jar.borrow_mut() = cookies);
    COOKIE_MUTATIONS.with(|mutations| mutations.borrow_mut().clear());
}

pub(crate) fn visible_pairs() -> Vec<(String, String)> {
    COOKIE_JAR.with(|cookies| cookies.borrow().clone())
}

pub(crate) fn mutations() -> Vec<String> {
    COOKIE_MUTATIONS.with(|mutations| mutations.borrow().clone())
}

pub(crate) fn cookie_string() -> String {
    visible_pairs()
        .into_iter()
        .map(|(name, value)| format!("{name}={value}"))
        .collect::<Vec<_>>()
        .join("; ")
}

pub(crate) fn set_document_cookie(raw: &str) {
    let Some((name, rest)) = raw.split_once('=') else {
        return;
    };
    let name = name.trim();
    if name.is_empty() {
        return;
    }
    COOKIE_MUTATIONS.with(|mutations| mutations.borrow_mut().push(raw.into()));
    update::visible(
        name,
        rest.split(';').next().unwrap_or_default().trim(),
        delete::deletes(raw),
    );
}
