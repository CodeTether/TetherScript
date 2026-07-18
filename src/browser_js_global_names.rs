//! Browser window properties promoted into the JavaScript global scope.

#[path = "browser_js_global_names/constructors.rs"]
mod constructors;
#[path = "browser_js_global_names/platform.rs"]
mod platform;
#[path = "browser_js_global_names/web.rs"]
mod web;

pub(super) fn all() -> impl Iterator<Item = &'static str> {
    constructors::ALL
        .iter()
        .chain(platform::ALL)
        .chain(web::ALL)
        .copied()
}
