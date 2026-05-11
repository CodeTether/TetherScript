//! Healing strategies.
use super::{DomFingerprint, DomNode, HealStrategy, SelectorCandidate, SelectorGenerator};

pub fn attr_recovery(root: &DomNode, path: &[usize], g: &SelectorGenerator) -> Vec<SelectorCandidate> {
    g.generate(root, path).into_iter().map(|s| {
        let m = g.find(root, &s);
        SelectorCandidate::new(s, if m.len() == 1 { 0.9 } else { 0.55 }, HealStrategy::AttrRecovery, m)
    }).collect()
}

pub fn structural_path(root: &DomNode, path: &[usize], g: &SelectorGenerator) -> Vec<SelectorCandidate> {
    let s = g.generate(root, path).last().cloned().unwrap_or_default();
    vec![SelectorCandidate::new(s, 0.45, HealStrategy::StructuralPath, vec![path.to_vec()])]
}

pub fn text_proximity(root: &DomNode, fp: &DomFingerprint) -> Vec<SelectorCandidate> {
    let mut all = Vec::new(); root.walk(&mut all, vec![]);
    all.into_iter().filter_map(|(p, n)| {
        let f = DomFingerprint::from_dom(root, &p)?;
        (f.tag == fp.tag && f.text_hash == fp.text_hash).then(|| {
            SelectorCandidate::new(format!("{}:text(\"{}\")", n.tag, n.text.trim()), 0.75, HealStrategy::TextProximity, vec![p])
        })
    }).collect()
}

pub fn role_based(root: &DomNode, path: &[usize], g: &SelectorGenerator) -> Vec<SelectorCandidate> {
    let Some(n) = root.get(path) else { return vec![] };
    let Some(role) = n.attr("role") else { return vec![] };
    let s = format!("[role=\"{}\"][aria-label=\"{}\"]", role, n.name());
    let m = g.find(root, &s);
    vec![SelectorCandidate::new(s, 0.82, HealStrategy::RoleBased, m)]
}

pub fn sibling_proximity(root: &DomNode, path: &[usize]) -> Vec<SelectorCandidate> {
    if path.is_empty() { return vec![] }
    let mut parent = path.to_vec(); let idx = parent.pop().unwrap();
    let Some(p) = root.get(&parent) else { return vec![] };
    let tag = p.children.get(idx).map(|n| n.tag.clone()).unwrap_or_default();
    vec![SelectorCandidate::new(format!("{}:near-sibling({})", tag, idx), 0.38, HealStrategy::SiblingProximity, vec![path.to_vec()])]
}

pub fn position_hint(root: &DomNode, path: &[usize]) -> Vec<SelectorCandidate> {
    let Some(n) = root.get(path) else { return vec![] };
    let pos = path.last().copied().unwrap_or(0) + 1;
    vec![SelectorCandidate::new(format!("the {}th {} in its parent", pos, n.tag), 0.25, HealStrategy::PositionHint, vec![path.to_vec()])]
}
