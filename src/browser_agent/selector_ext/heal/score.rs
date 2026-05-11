//! Element similarity scoring.

/// Properties fingerprint of an element for similarity matching.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ElementProps {
    pub tag: String,
    pub id: Option<String>,
    pub classes: Vec<String>,
    pub role: Option<String>,
    pub text: Option<String>,
    pub href: Option<String>,
    pub label: Option<String>,
    pub parent_tag: Option<String>,
    pub position_hint: Option<usize>,
}

/// Compute a similarity score between 0.0 and 1.0 for two elements.
pub fn element_similarity(a: &ElementProps, b: &ElementProps) -> f64 {
    let mut s = 0.0;
    if clean(&a.tag) == clean(&b.tag) { s += 0.20; }
    if opt_eq(&a.id, &b.id) { s += 0.30; }
    s += class_overlap(&a.classes, &b.classes) * 0.20;
    if text_eq(&a.text, &b.text) || text_eq(&a.label, &b.label) { s += 0.15; }
    if opt_eq(&a.role, &b.role) { s += 0.10; }
    if a.position_hint.is_some() && a.position_hint == b.position_hint { s += 0.05; }
    s.min(1.0)
}

fn clean(s: &str) -> String { s.trim().to_ascii_lowercase() }
fn opt_eq(a: &Option<String>, b: &Option<String>) -> bool {
    match (a, b) { (Some(a), Some(b)) => clean(a) == clean(b), _ => false }
}
fn text_eq(a: &Option<String>, b: &Option<String>) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => { let a = clean(a); let b = clean(b); !a.is_empty() && (a == b || a.contains(&b) || b.contains(&a)) }
        _ => false,
    }
}
fn class_overlap(a: &[String], b: &[String]) -> f64 {
    if a.is_empty() || b.is_empty() { return 0.0; }
    let hit = a.iter().filter(|x| b.iter().any(|y| clean(x) == clean(y))).count();
    hit as f64 / a.len().max(b.len()) as f64
}
