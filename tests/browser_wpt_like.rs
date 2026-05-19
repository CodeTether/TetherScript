#[path = "browser_wpt_like/mod.rs"]
mod fixtures;

#[test]
fn dom_events_fixture_subset() {
    fixtures::dom_events::run();
}

#[test]
fn selectors_fixture_subset() {
    fixtures::selectors::run();
}

#[test]
fn fetch_cors_fixture_subset() {
    fixtures::fetch_cors::run();
}

#[test]
fn modules_fixture_subset() {
    fixtures::modules::run();
}

#[test]
fn css_layout_fixture_subset() {
    fixtures::css_layout::run();
}

#[test]
fn timers_microtasks_fixture_subset() {
    fixtures::timers_microtasks::run();
}

#[test]
fn storage_fixture_subset() {
    fixtures::storage::run();
}

#[test]
fn html_tree_fixture_subset() {
    fixtures::html_tree::run();
}
