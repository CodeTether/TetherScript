//! Coarse device classification from `User-Agent`.
//!
//! # Three classes only
//!
//! `mobile`, `tablet`, `desktop`. A small, stable domain is the point: the class
//! becomes a cache-key component, so every extra class divides the cache again.
//! Three is what a responsive template can actually branch on.
//!
//! # Tablet is tested first
//!
//! An iPad reports `Mozilla/5.0 (iPad; CPU OS 17_0 like Mac OS X) … Mobile/15E148`
//! and Android tablets report `Android … Tablet`, so the string carries *both*
//! signals. Testing `mobile` first would classify every tablet as a phone, which is
//! the single most common bug in this kind of function. Tablet therefore wins.
//!
//! # A guess, never a security boundary
//!
//! `User-Agent` is attacker-controlled and trivially spoofed, so nothing may gate
//! access on this result: a forged header is a request for a different *layout*,
//! which is harmless. The return value is one of three fixed literals and never any
//! part of the header, so nothing attacker-supplied is echoed into a cache key or a
//! response. That also bounds the work — only membership tests run, over the
//! caller's own header, with no allocation beyond one lowercase copy.

use std::collections::HashMap;

use super::dynpage_request::find;
use crate::value::Value;

/// Substrings that indicate a tablet. Tested before the phone markers.
const TABLET: [&str; 4] = ["ipad", "tablet", "kindle", "playbook"];

/// Substrings that indicate a phone.
const MOBILE: [&str; 5] = ["iphone", "ipod", "android", "windows phone", "mobile"];

/// Classify the client device from its `User-Agent`.
///
/// # Arguments
///
/// * `headers` — The request's header map.
///
/// # Returns
///
/// `"tablet"`, `"mobile"`, or `"desktop"`. An absent, empty, or unrecognised
/// `User-Agent` is `desktop`, the safest default: it is the fullest layout, so a
/// misclassified client sees too much rather than too little.
pub(super) fn classify(headers: &HashMap<String, Value>) -> &'static str {
    let Some(agent) = find(headers, "user-agent") else {
        return "desktop";
    };
    let agent = agent.to_ascii_lowercase();
    if TABLET.iter().any(|mark| agent.contains(*mark)) {
        return "tablet";
    }
    if MOBILE.iter().any(|mark| agent.contains(*mark)) {
        return "mobile";
    }
    "desktop"
}
