//! DOM element fingerprints.
use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};

/// Minimal DOM node used by the selector healer.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DomNode {
    pub tag: String,
    pub text: String,
    pub attrs: BTreeMap<String, String>,
    pub children: Vec<DomNode>,
}

impl DomNode {
    pub fn attr(&self, k: &str) -> Option<&str> {
        self.attrs.get(k).map(String::as_str)
    }

    pub fn name(&self) -> String {
        self.attr("aria-label")
            .or_else(|| self.attr("title"))
            .map(str::to_string)
            .unwrap_or_else(|| self.text.trim().to_string())
    }

    pub fn get<'a>(&'a self, path: &[usize]) -> Option<&'a DomNode> {
        let mut n = self;
        for &i in path { n = n.children.get(i)?; }
        Some(n)
    }

    pub fn walk(&self, out: &mut Vec<(Vec<usize>, DomNode)>, path: Vec<usize>) {
        out.push((path.clone(), self.clone()));
        for (i, c) in self.children.iter().enumerate() {
            let mut p = path.clone();
            p.push(i);
            c.walk(out, p);
        }
    }
}

/// Identity fingerprint for matching an element across DOM changes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DomFingerprint {
    pub tag: String,
    pub text_hash: u64,
    pub attr_hash: u64,
    pub structural_hash: u64,
}

impl DomFingerprint {
    pub fn from_dom(root: &DomNode, path: &[usize]) -> Option<Self> {
        let n = root.get(path)?;
        Some(Self {
            tag: n.tag.clone(),
            text_hash: hash_str(&norm(&n.text)),
            attr_hash: hash_str(&stable_attrs(n)),
            structural_hash: hash_str(&path.iter().map(usize::to_string).collect::<Vec<_>>().join("/")),
        })
    }

    pub fn similarity(&self, other: &Self) -> f32 {
        let mut s = 0.0;
        if self.tag == other.tag { s += 0.30; }
        if self.text_hash == other.text_hash { s += 0.30; }
        if self.attr_hash == other.attr_hash { s += 0.25; }
        if self.structural_hash == other.structural_hash { s += 0.15; }
        s
    }
}

fn norm(s: &str) -> String { s.split_whitespace().collect::<Vec<_>>().join(" ") }
fn stable_attrs(n: &DomNode) -> String {
    ["data-testid", "aria-label", "role", "name", "type", "href"]
        .iter().filter_map(|k| n.attr(k).map(|v| format!("{k}={v}")))
        .collect::<Vec<_>>().join("|")
}
fn hash_str<T: Hash>(v: &T) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    v.hash(&mut h); h.finish()
}
