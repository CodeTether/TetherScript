//! Robust selector generation.
use super::{DomNode, ElementPath};

#[derive(Clone, Debug, Default)]
pub struct SelectorGenerator;

impl SelectorGenerator {
    pub fn generate(&self, root: &DomNode, path: &[usize]) -> Vec<String> {
        let Some(n) = root.get(path) else { return vec![] };
        let mut v = Vec::new();
        for a in ["data-testid", "data-test", "aria-label", "name"] {
            if let Some(x) = n.attr(a) { v.push(format!("[{a}=\"{}\"]", esc(x))); }
        }
        if let Some(role) = n.attr("role") {
            let name = n.name();
            if !name.is_empty() { v.push(format!("[role=\"{}\"][aria-label=\"{}\"]", esc(role), esc(&name))); }
            v.push(format!("[role=\"{}\"]", esc(role)));
        }
        if let Some(id) = n.attr("id").filter(|x| !fragile_id(x)) { v.push(format!("#{}", id)); }
        if let Some(c) = unique_class(n) { v.push(format!("{}.{}", n.tag, c)); }
        if !n.text.trim().is_empty() { v.push(format!("{}:text(\"{}\")", n.tag, esc(n.text.trim()))); }
        v.push(self.structural(root, path));
        v.into_iter().filter(|s| !s.is_empty()).collect()
    }

    pub fn shortest_unique(&self, root: &DomNode, path: &[usize]) -> Option<String> {
        let mut xs = self.generate(root, path);
        xs.sort_by_key(String::len);
        xs.into_iter().find(|s| self.find(root, s).len() == 1)
    }

    pub fn find(&self, root: &DomNode, selector: &str) -> Vec<ElementPath> {
        let mut all = Vec::new(); root.walk(&mut all, vec![]);
        all.into_iter().filter_map(|(p, n)| matches_sel(&n, selector).then_some(p)).collect()
    }

    fn structural(&self, root: &DomNode, path: &[usize]) -> String {
        let mut cur = vec![]; let mut parts = vec![];
        for &i in path {
            let Some(n) = root.get(&cur) else { break };
            let tag = n.children.get(i).map(|c| c.tag.as_str()).unwrap_or("*");
            parts.push(format!("{}:nth-child({})", tag, i + 1));
            cur.push(i);
        }
        parts.join(" > ")
    }
}

fn esc(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}
fn fragile_id(s: &str) -> bool { s.chars().filter(|c| c.is_ascii_digit()).count() > 4 || s.len() > 32 }
fn unique_class(n: &DomNode) -> Option<String> {
    n.attr("class")?.split_whitespace()
        .find(|c| !c.contains('_') && !c.chars().any(|x| x.is_ascii_digit()))
        .map(str::to_string)
}
fn matches_sel(n: &DomNode, s: &str) -> bool {
    if let Some(id) = s.strip_prefix('#') { return n.attr("id") == Some(id) }
    if s.contains(":text(\"") { return s.starts_with(&n.tag) && s.contains(n.text.trim()) }
    if let Some(x) = s.strip_prefix('[').and_then(|x| x.strip_suffix(']')) {
        let mut p = x.splitn(2, "=\""); let k = p.next().unwrap_or("");
        let v = p.next().unwrap_or("").trim_end_matches('"');
        return n.attr(k) == Some(v);
    }
    s == n.tag
}
