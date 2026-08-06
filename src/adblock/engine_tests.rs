//! Tests for the combined network + cosmetic ad-block engine.
//!
//! `Engine` is the stateful convenience layer over the stateless `adblock_*`
//! built-ins: it holds compiled rules and running block/allow counters. These
//! tests are its only consumer inside the crate, so they also keep the layer
//! honest about compiling.

use super::engine::Engine;

fn engine_with(list: &str) -> Engine {
    let mut engine = Engine::new();
    engine.add_list(list);
    engine
}

#[test]
fn a_new_engine_has_no_rules_and_no_counts() {
    let engine = Engine::new();

    assert_eq!(engine.rule_count(), 0);
    assert_eq!(engine.blocked_count, 0);
    assert_eq!(engine.allowed_count, 0);
}

#[test]
fn comments_are_not_counted_as_rules() {
    let engine = engine_with("! a comment\n||ads.example.com^\n");

    assert_eq!(engine.rule_count(), 1);
}

#[test]
fn a_matching_request_is_blocked_and_counted() {
    let mut engine = engine_with("||ads.example.com^");

    assert!(engine.should_block("https://ads.example.com/banner.gif", "site.com"));
    assert_eq!(engine.blocked_count, 1);
    assert_eq!(engine.allowed_count, 0);
}

#[test]
fn an_unmatched_request_is_allowed_and_counted() {
    let mut engine = engine_with("||ads.example.com^");

    assert!(!engine.should_block("https://cdn.site.com/app.js", "site.com"));
    assert_eq!(engine.blocked_count, 0);
    assert_eq!(engine.allowed_count, 1);
}

#[test]
fn counters_accumulate_across_requests() {
    let mut engine = engine_with("||ads.example.com^");

    engine.should_block("https://ads.example.com/a.gif", "site.com");
    engine.should_block("https://ads.example.com/b.gif", "site.com");
    engine.should_block("https://site.com/ok.js", "site.com");

    assert_eq!(engine.blocked_count, 2);
    assert_eq!(engine.allowed_count, 1);
}

#[test]
fn cosmetic_selectors_are_returned_for_a_matching_domain() {
    let engine = engine_with("site.com##.ad-banner");

    assert_eq!(engine.cosmetic_selectors("site.com"), vec![".ad-banner"]);
}

#[test]
fn a_default_engine_matches_a_new_one() {
    assert_eq!(Engine::default().rule_count(), Engine::new().rule_count());
}
