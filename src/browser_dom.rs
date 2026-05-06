//! Arena-backed DOM representation for browser experiments.
//!
//! `browser::Document` is a recursive tree, which is convenient for parsing and
//! snapshots but awkward for mutation-oriented browser work. `DomArena` stores
//! nodes in a stable-id arena while preserving parent/child links.

use std::collections::HashMap;

use crate::browser;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeId(usize);

impl NodeId {
    pub fn index(self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomNodeKind {
    Element(DomElement),
    Text(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DomElement {
    pub tag: String,
    pub attrs: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArenaNode {
    pub parent: Option<NodeId>,
    pub children: Vec<NodeId>,
    pub kind: DomNodeKind,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DomArena {
    nodes: Vec<ArenaNode>,
    roots: Vec<NodeId>,
}

impl DomArena {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_document(document: &browser::Document) -> Self {
        let mut arena = Self::new();
        for node in &document.children {
            arena.push_browser_node(None, node);
        }
        arena
    }

    pub fn to_document(&self) -> browser::Document {
        browser::Document {
            children: self
                .roots
                .iter()
                .map(|root| self.to_browser_node(*root))
                .collect(),
        }
    }

    pub fn add_element(
        &mut self,
        parent: Option<NodeId>,
        tag: impl Into<String>,
        attrs: HashMap<String, String>,
    ) -> NodeId {
        self.push_node(
            parent,
            DomNodeKind::Element(DomElement {
                tag: tag.into(),
                attrs,
            }),
        )
    }

    pub fn add_text(&mut self, parent: Option<NodeId>, text: impl Into<String>) -> NodeId {
        self.push_node(parent, DomNodeKind::Text(text.into()))
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn root_ids(&self) -> &[NodeId] {
        &self.roots
    }

    pub fn node(&self, id: NodeId) -> Option<&ArenaNode> {
        self.nodes.get(id.0)
    }

    pub fn parent(&self, id: NodeId) -> Option<NodeId> {
        self.node(id).and_then(|node| node.parent)
    }

    pub fn children(&self, id: NodeId) -> Option<&[NodeId]> {
        self.node(id).map(|node| node.children.as_slice())
    }

    pub fn kind(&self, id: NodeId) -> Option<&DomNodeKind> {
        self.node(id).map(|node| &node.kind)
    }

    pub fn element(&self, id: NodeId) -> Option<&DomElement> {
        match self.kind(id)? {
            DomNodeKind::Element(element) => Some(element),
            DomNodeKind::Text(_) => None,
        }
    }

    pub fn text(&self, id: NodeId) -> Option<&str> {
        match self.kind(id)? {
            DomNodeKind::Element(_) => None,
            DomNodeKind::Text(text) => Some(text),
        }
    }

    /// Query this arena with the existing recursive DOM selector engine.
    ///
    /// The adapter converts the arena back into `browser::Document`, delegates to
    /// `browser::query_selector`, then maps matched recursive nodes back to arena
    /// ids by pre-order structural equality. If the document contains identical
    /// duplicate subtrees, matches are assigned to the first still-unclaimed equal
    /// arena nodes in document order.
    pub fn query_selector(&self, selector: &str) -> Vec<NodeId> {
        let matches = browser::query_selector(&self.to_document(), selector);
        let mut claimed = vec![false; self.nodes.len()];
        matches
            .iter()
            .filter_map(|matched| self.find_unclaimed_browser_node(matched, &mut claimed))
            .collect()
    }

    fn push_browser_node(&mut self, parent: Option<NodeId>, node: &browser::Node) -> NodeId {
        match node {
            browser::Node::Element(element) => {
                let id = self.add_element(parent, element.tag.clone(), element.attrs.clone());
                for child in &element.children {
                    self.push_browser_node(Some(id), child);
                }
                id
            }
            browser::Node::Text(text) => self.add_text(parent, text.clone()),
        }
    }

    fn push_node(&mut self, parent: Option<NodeId>, kind: DomNodeKind) -> NodeId {
        let id = NodeId(self.nodes.len());
        self.nodes.push(ArenaNode {
            parent,
            children: Vec::new(),
            kind,
        });
        if let Some(parent) = parent {
            self.nodes[parent.0].children.push(id);
        } else {
            self.roots.push(id);
        }
        id
    }

    fn to_browser_node(&self, id: NodeId) -> browser::Node {
        let node = &self.nodes[id.0];
        match &node.kind {
            DomNodeKind::Element(element) => browser::Node::Element(browser::Element {
                tag: element.tag.clone(),
                attrs: element.attrs.clone(),
                children: node
                    .children
                    .iter()
                    .map(|child| self.to_browser_node(*child))
                    .collect(),
            }),
            DomNodeKind::Text(text) => browser::Node::Text(text.clone()),
        }
    }

    fn find_unclaimed_browser_node(
        &self,
        matched: &browser::Node,
        claimed: &mut [bool],
    ) -> Option<NodeId> {
        self.roots
            .iter()
            .find_map(|root| self.find_unclaimed_from(*root, matched, claimed))
    }

    fn find_unclaimed_from(
        &self,
        id: NodeId,
        matched: &browser::Node,
        claimed: &mut [bool],
    ) -> Option<NodeId> {
        if !claimed[id.0] && self.browser_node_eq(id, matched) {
            claimed[id.0] = true;
            return Some(id);
        }
        self.nodes[id.0]
            .children
            .iter()
            .find_map(|child| self.find_unclaimed_from(*child, matched, claimed))
    }

    fn browser_node_eq(&self, id: NodeId, other: &browser::Node) -> bool {
        match (&self.nodes[id.0].kind, other) {
            (DomNodeKind::Text(left), browser::Node::Text(right)) => left == right,
            (DomNodeKind::Element(left), browser::Node::Element(right)) => {
                left.tag == right.tag
                    && left.attrs == right.attrs
                    && self.nodes[id.0].children.len() == right.children.len()
                    && self.nodes[id.0].children.iter().zip(&right.children).all(
                        |(left_child, right_child)| self.browser_node_eq(*left_child, right_child),
                    )
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_document_to_parent_child_arena() {
        let document = browser::parse_html(r#"<main id="app"><h1>Hello</h1><p>World</p></main>"#);
        let arena = DomArena::from_document(&document);

        assert_eq!(arena.len(), 5);
        let main = arena.root_ids()[0];
        assert_eq!(arena.parent(main), None);
        assert_eq!(arena.element(main).unwrap().tag, "main");
        assert_eq!(arena.element(main).unwrap().attrs.get("id").unwrap(), "app");

        let children = arena.children(main).unwrap();
        assert_eq!(children.len(), 2);
        assert_eq!(arena.parent(children[0]), Some(main));
        assert_eq!(arena.element(children[0]).unwrap().tag, "h1");
        assert_eq!(arena.element(children[1]).unwrap().tag, "p");
    }

    #[test]
    fn round_trips_back_to_recursive_document() {
        let document = browser::parse_html(
            r#"<section class="panel"><p data-kind="intro">Hello <em>Rust</em></p></section>"#,
        );
        let arena = DomArena::from_document(&document);

        assert_eq!(arena.to_document(), document);
    }

    #[test]
    fn query_selector_returns_matching_node_ids() {
        let document = browser::parse_html(
            r#"<div id="app"><p class="lead">Intro</p><p data-kind="body">Body</p></div>"#,
        );
        let arena = DomArena::from_document(&document);

        let lead = arena.query_selector("#app > p.lead");
        assert_eq!(lead.len(), 1);
        assert_eq!(arena.element(lead[0]).unwrap().tag, "p");
        assert_eq!(
            arena.text(arena.children(lead[0]).unwrap()[0]),
            Some("Intro")
        );

        let body = arena.query_selector(r#"p[data-kind="body"]"#);
        assert_eq!(body.len(), 1);
        assert_eq!(
            arena.text(arena.children(body[0]).unwrap()[0]),
            Some("Body")
        );
    }

    #[test]
    fn duplicate_subtrees_map_to_distinct_ids_in_document_order() {
        let document = browser::parse_html(r#"<ul><li>Same</li><li>Same</li></ul>"#);
        let arena = DomArena::from_document(&document);

        let items = arena.query_selector("li");
        assert_eq!(items.len(), 2);
        assert_ne!(items[0], items[1]);
        assert!(items[0] < items[1]);
    }
}
