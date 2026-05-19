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
