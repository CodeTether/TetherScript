//! Experimental browser/runtime foundation.
//!
//! This is deliberately small, but it is a real end-to-end slice: HTML is parsed
//! into a DOM tree, CSS rules are parsed and matched, a block-flow layout tree is
//! computed, and a text display-list renderer produces deterministic output. It
//! is not web-compatible yet; it is the seed for the browser track.

use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Write as _;
use std::rc::Rc;

use crate::value::Value;

#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    Element(Element),
    Text(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Element {
    pub tag: String,
    pub attrs: HashMap<String, String>,
    pub children: Vec<Node>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    pub children: Vec<Node>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CssRule {
    pub selector: Selector,
    pub declarations: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Selector {
    Tag(String),
    Class(String),
    Id(String),
    Universal,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StyledNode {
    pub node: Node,
    pub styles: HashMap<String, String>,
    pub children: Vec<StyledNode>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LayoutBox {
    pub kind: String,
    pub tag: Option<String>,
    pub text: Option<String>,
    pub x: i64,
    pub y: i64,
    pub width: i64,
    pub height: i64,
    pub styles: HashMap<String, String>,
    pub children: Vec<LayoutBox>,
}

pub fn parse_html(source: &str) -> Document {
    HtmlParser::new(source).parse_document()
}

pub fn parse_css(source: &str) -> Vec<CssRule> {
    let mut rules = Vec::new();
    for part in source.split('}') {
        let Some((selector_raw, body)) = part.split_once('{') else {
            continue;
        };
        let selector_raw = selector_raw.trim();
        if selector_raw.is_empty() {
            continue;
        }
        let selector = if let Some(id) = selector_raw.strip_prefix('#') {
            Selector::Id(id.trim().to_string())
        } else if let Some(class) = selector_raw.strip_prefix('.') {
            Selector::Class(class.trim().to_string())
        } else if selector_raw == "*" {
            Selector::Universal
        } else {
            Selector::Tag(selector_raw.to_ascii_lowercase())
        };
        let mut declarations = HashMap::new();
        for decl in body.split(';') {
            let Some((name, value)) = decl.split_once(':') else {
                continue;
            };
            let name = name.trim().to_ascii_lowercase();
            let value = value.trim().to_string();
            if !name.is_empty() && !value.is_empty() {
                declarations.insert(name, value);
            }
        }
        rules.push(CssRule {
            selector,
            declarations,
        });
    }
    rules
}

pub fn style_document(document: &Document, rules: &[CssRule]) -> Vec<StyledNode> {
    document
        .children
        .iter()
        .map(|node| style_node(node, rules))
        .collect()
}

pub fn layout_document(document: &Document, css: &str, width: i64) -> LayoutBox {
    let rules = parse_css(css);
    let styled = style_document(document, &rules);
    let mut root = LayoutBox {
        kind: "viewport".into(),
        tag: None,
        text: None,
        x: 0,
        y: 0,
        width: width.max(1),
        height: 0,
        styles: HashMap::new(),
        children: Vec::new(),
    };
    let mut cursor_y = 0;
    for child in styled {
        let layout = layout_styled_node(&child, 0, cursor_y, root.width);
        cursor_y += layout.height;
        root.children.push(layout);
    }
    root.height = cursor_y.max(1);
    root
}

pub fn render_text(layout: &LayoutBox) -> String {
    let mut out = String::new();
    render_box(layout, 0, &mut out);
    out
}

pub fn document_to_value(document: &Document) -> Value {
    let children = document.children.iter().map(node_to_value).collect();
    let mut map = HashMap::new();
    map.insert("type".into(), Value::Str(Rc::new("document".into())));
    map.insert(
        "children".into(),
        Value::List(Rc::new(RefCell::new(children))),
    );
    Value::Map(Rc::new(RefCell::new(map)))
}

pub fn layout_to_value(layout: &LayoutBox) -> Value {
    let mut map = HashMap::new();
    map.insert("kind".into(), Value::Str(Rc::new(layout.kind.clone())));
    map.insert("x".into(), Value::Int(layout.x));
    map.insert("y".into(), Value::Int(layout.y));
    map.insert("width".into(), Value::Int(layout.width));
    map.insert("height".into(), Value::Int(layout.height));
    map.insert(
        "tag".into(),
        layout
            .tag
            .as_ref()
            .map(|tag| Value::Str(Rc::new(tag.clone())))
            .unwrap_or(Value::Nil),
    );
    map.insert(
        "text".into(),
        layout
            .text
            .as_ref()
            .map(|text| Value::Str(Rc::new(text.clone())))
            .unwrap_or(Value::Nil),
    );
    map.insert("styles".into(), string_map_to_value(&layout.styles));
    let children = layout.children.iter().map(layout_to_value).collect();
    map.insert(
        "children".into(),
        Value::List(Rc::new(RefCell::new(children))),
    );
    Value::Map(Rc::new(RefCell::new(map)))
}

fn style_node(node: &Node, rules: &[CssRule]) -> StyledNode {
    let mut styles = HashMap::new();
    if let Node::Element(element) = node {
        for rule in rules {
            if selector_matches(&rule.selector, element) {
                for (name, value) in &rule.declarations {
                    styles.insert(name.clone(), value.clone());
                }
            }
        }
    }
    let children = match node {
        Node::Element(element) => element
            .children
            .iter()
            .map(|child| style_node(child, rules))
            .collect(),
        Node::Text(_) => Vec::new(),
    };
    StyledNode {
        node: node.clone(),
        styles,
        children,
    }
}

fn selector_matches(selector: &Selector, element: &Element) -> bool {
    match selector {
        Selector::Universal => true,
        Selector::Tag(tag) => &element.tag == tag,
        Selector::Id(id) => element.attrs.get("id") == Some(id),
        Selector::Class(class) => element
            .attrs
            .get("class")
            .is_some_and(|classes| classes.split_whitespace().any(|item| item == class)),
    }
}

fn layout_styled_node(styled: &StyledNode, x: i64, y: i64, containing_width: i64) -> LayoutBox {
    match &styled.node {
        Node::Text(text) => {
            let width = text.chars().count() as i64;
            LayoutBox {
                kind: "text".into(),
                tag: None,
                text: Some(text.clone()),
                x,
                y,
                width: width.min(containing_width).max(0),
                height: if text.trim().is_empty() { 0 } else { 1 },
                styles: styled.styles.clone(),
                children: Vec::new(),
            }
        }
        Node::Element(element) => {
            if styled.styles.get("display").is_some_and(|v| v == "none") {
                return LayoutBox {
                    kind: "none".into(),
                    tag: Some(element.tag.clone()),
                    text: None,
                    x,
                    y,
                    width: 0,
                    height: 0,
                    styles: styled.styles.clone(),
                    children: Vec::new(),
                };
            }
            let width = parse_px(styled.styles.get("width"))
                .unwrap_or(containing_width)
                .max(1);
            let padding = parse_px(styled.styles.get("padding")).unwrap_or(0).max(0);
            let mut cursor_y = y + padding;
            let mut children = Vec::new();
            for child in &styled.children {
                let child_layout =
                    layout_styled_node(child, x + padding, cursor_y, (width - padding * 2).max(1));
                cursor_y += child_layout.height;
                if child_layout.height > 0 || child_layout.kind != "none" {
                    children.push(child_layout);
                }
            }
            let explicit_height = parse_px(styled.styles.get("height"));
            let content_height = (cursor_y - y) + padding;
            let height = explicit_height.unwrap_or(content_height.max(1));
            LayoutBox {
                kind: "block".into(),
                tag: Some(element.tag.clone()),
                text: None,
                x,
                y,
                width,
                height,
                styles: styled.styles.clone(),
                children,
            }
        }
    }
}

fn parse_px(value: Option<&String>) -> Option<i64> {
    let value = value?.trim();
    let value = value.strip_suffix("px").unwrap_or(value).trim();
    value.parse::<i64>().ok()
}

fn render_box(layout: &LayoutBox, indent: usize, out: &mut String) {
    let pad = "  ".repeat(indent);
    match layout.kind.as_str() {
        "viewport" => {
            let _ = writeln!(out, "viewport {}x{}", layout.width, layout.height);
        }
        "block" => {
            let tag = layout.tag.as_deref().unwrap_or("block");
            let _ = writeln!(
                out,
                "{}<{}> @{},{} {}x{}",
                pad, tag, layout.x, layout.y, layout.width, layout.height
            );
        }
        "text" => {
            if let Some(text) = &layout.text {
                if !text.trim().is_empty() {
                    let _ = writeln!(
                        out,
                        "{}\"{}\" @{},{} {}x{}",
                        pad,
                        text.trim(),
                        layout.x,
                        layout.y,
                        layout.width,
                        layout.height
                    );
                }
            }
        }
        _ => {}
    }
    for child in &layout.children {
        render_box(child, indent + 1, out);
    }
}

fn node_to_value(node: &Node) -> Value {
    let mut map = HashMap::new();
    match node {
        Node::Text(text) => {
            map.insert("type".into(), Value::Str(Rc::new("text".into())));
            map.insert("text".into(), Value::Str(Rc::new(text.clone())));
        }
        Node::Element(element) => {
            map.insert("type".into(), Value::Str(Rc::new("element".into())));
            map.insert("tag".into(), Value::Str(Rc::new(element.tag.clone())));
            map.insert("attrs".into(), string_map_to_value(&element.attrs));
            let children = element.children.iter().map(node_to_value).collect();
            map.insert(
                "children".into(),
                Value::List(Rc::new(RefCell::new(children))),
            );
        }
    }
    Value::Map(Rc::new(RefCell::new(map)))
}

fn string_map_to_value(input: &HashMap<String, String>) -> Value {
    let map = input
        .iter()
        .map(|(key, value)| (key.clone(), Value::Str(Rc::new(value.clone()))))
        .collect();
    Value::Map(Rc::new(RefCell::new(map)))
}

struct HtmlParser<'a> {
    src: &'a str,
    pos: usize,
}

impl<'a> HtmlParser<'a> {
    fn new(src: &'a str) -> Self {
        Self { src, pos: 0 }
    }

    fn parse_document(&mut self) -> Document {
        Document {
            children: self.parse_nodes(None),
        }
    }

    fn parse_nodes(&mut self, until: Option<&str>) -> Vec<Node> {
        let mut nodes = Vec::new();
        while !self.eof() {
            if self.starts_with("<!--") {
                self.consume_comment();
                continue;
            }
            if self.starts_with("</") {
                let tag = self.peek_closing_tag();
                self.consume_until('>');
                self.consume_char();
                if until.is_none_or(|expected| tag == expected) {
                    break;
                }
                continue;
            }
            if self.starts_with("<") {
                if let Some(node) = self.parse_element() {
                    nodes.push(node);
                }
            } else {
                let text = decode_entities(&self.consume_text());
                if !text.is_empty() {
                    nodes.push(Node::Text(text));
                }
            }
        }
        nodes
    }

    fn parse_element(&mut self) -> Option<Node> {
        self.consume_char();
        self.consume_whitespace();
        let tag = self
            .consume_while(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == ':')
            .to_ascii_lowercase();
        if tag.is_empty() {
            self.consume_until('>');
            self.consume_char();
            return None;
        }
        let attrs = self.parse_attrs();
        self.consume_whitespace();
        let self_closing = self.starts_with("/");
        if self_closing {
            self.consume_char();
        }
        if self.starts_with(">") {
            self.consume_char();
        }
        let children = if self_closing || is_void_element(&tag) {
            Vec::new()
        } else {
            self.parse_nodes(Some(&tag))
        };
        Some(Node::Element(Element {
            tag,
            attrs,
            children,
        }))
    }

    fn parse_attrs(&mut self) -> HashMap<String, String> {
        let mut attrs = HashMap::new();
        loop {
            self.consume_whitespace();
            if self.eof() || self.starts_with(">") || self.starts_with("/") {
                break;
            }
            let name = self
                .consume_while(|ch| {
                    ch.is_ascii_alphanumeric() || ch == '-' || ch == ':' || ch == '_'
                })
                .to_ascii_lowercase();
            if name.is_empty() {
                self.consume_char();
                continue;
            }
            self.consume_whitespace();
            let value = if self.starts_with("=") {
                self.consume_char();
                self.consume_whitespace();
                self.consume_attr_value()
            } else {
                String::new()
            };
            attrs.insert(name, decode_entities(&value));
        }
        attrs
    }

    fn consume_attr_value(&mut self) -> String {
        match self.next_char() {
            Some('"') | Some('\'') => {
                let quote = self.consume_char().unwrap();
                let value = self.consume_while(|ch| ch != quote);
                self.consume_char();
                value
            }
            _ => self.consume_while(|ch| !ch.is_whitespace() && ch != '>'),
        }
    }

    fn consume_text(&mut self) -> String {
        self.consume_while(|ch| ch != '<')
    }

    fn consume_comment(&mut self) {
        if let Some(end) = self.src[self.pos..].find("-->") {
            self.pos += end + 3;
        } else {
            self.pos = self.src.len();
        }
    }

    fn peek_closing_tag(&self) -> String {
        let rest = self.src[self.pos + 2..].trim_start();
        rest.chars()
            .take_while(|ch| ch.is_ascii_alphanumeric() || *ch == '-' || *ch == ':')
            .collect::<String>()
            .to_ascii_lowercase()
    }

    fn consume_until(&mut self, target: char) {
        while !self.eof() && self.next_char() != Some(target) {
            self.consume_char();
        }
    }

    fn starts_with(&self, s: &str) -> bool {
        self.src[self.pos..].starts_with(s)
    }

    fn eof(&self) -> bool {
        self.pos >= self.src.len()
    }

    fn next_char(&self) -> Option<char> {
        self.src[self.pos..].chars().next()
    }

    fn consume_char(&mut self) -> Option<char> {
        let ch = self.next_char()?;
        self.pos += ch.len_utf8();
        Some(ch)
    }

    fn consume_while<F>(&mut self, mut pred: F) -> String
    where
        F: FnMut(char) -> bool,
    {
        let mut out = String::new();
        while let Some(ch) = self.next_char() {
            if !pred(ch) {
                break;
            }
            out.push(ch);
            self.consume_char();
        }
        out
    }

    fn consume_whitespace(&mut self) {
        self.consume_while(char::is_whitespace);
    }
}

fn decode_entities(input: &str) -> String {
    input
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&amp;", "&")
}

fn is_void_element(tag: &str) -> bool {
    matches!(
        tag,
        "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "source"
            | "track"
            | "wbr"
    )
}

pub fn html_to_value(args: &[Value]) -> Result<Value, String> {
    let source = expect_str(args.first(), "browser_parse_html")?;
    Ok(document_to_value(&parse_html(source)))
}

pub fn render_to_value(args: &[Value]) -> Result<Value, String> {
    let html = expect_str(args.first(), "browser_render")?;
    let css = match args.get(1) {
        Some(Value::Str(css)) => css.as_str(),
        Some(other) => {
            return Err(format!(
                "browser_render: css must be str, got {}",
                other.type_name()
            ))
        }
        None => "",
    };
    let width = match args.get(2) {
        Some(Value::Int(width)) => *width,
        Some(other) => {
            return Err(format!(
                "browser_render: width must be int, got {}",
                other.type_name()
            ))
        }
        None => 80,
    };
    let doc = parse_html(html);
    let layout = layout_document(&doc, css, width);
    Ok(Value::Str(Rc::new(render_text(&layout))))
}

pub fn layout_to_runtime_value(args: &[Value]) -> Result<Value, String> {
    let html = expect_str(args.first(), "browser_layout")?;
    let css = match args.get(1) {
        Some(Value::Str(css)) => css.as_str(),
        Some(other) => {
            return Err(format!(
                "browser_layout: css must be str, got {}",
                other.type_name()
            ))
        }
        None => "",
    };
    let width = match args.get(2) {
        Some(Value::Int(width)) => *width,
        Some(other) => {
            return Err(format!(
                "browser_layout: width must be int, got {}",
                other.type_name()
            ))
        }
        None => 80,
    };
    let doc = parse_html(html);
    Ok(layout_to_value(&layout_document(&doc, css, width)))
}

fn expect_str<'a>(value: Option<&'a Value>, name: &str) -> Result<&'a str, String> {
    match value {
        Some(Value::Str(s)) => Ok(s.as_str()),
        Some(other) => Err(format!("{}: expected str, got {}", name, other.type_name())),
        None => Err(format!("{}: expected html string", name)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_html_to_dom_value() {
        let doc = parse_html(
            r#"<main id="app"><h1>Hello &amp; hi</h1><br><p class="note">World</p></main>"#,
        );
        assert_eq!(doc.children.len(), 1);
        match &doc.children[0] {
            Node::Element(el) => {
                assert_eq!(el.tag, "main");
                assert_eq!(el.attrs.get("id"), Some(&"app".to_string()));
                assert_eq!(el.children.len(), 3);
            }
            other => panic!("expected element, got {other:?}"),
        }
    }

    #[test]
    fn lays_out_and_renders_text_display_list() {
        let doc = parse_html(r#"<div class="card"><h1>Title</h1><p>Hello</p></div>"#);
        let layout = layout_document(
            &doc,
            ".card { width: 20px; padding: 1px } h1 { height: 2px }",
            80,
        );
        let rendered = render_text(&layout);
        assert!(rendered.contains("viewport 80x"));
        assert!(rendered.contains("<div> @0,0 20x"));
        assert!(rendered.contains("\"Title\""));
    }

    #[test]
    fn browser_builtins_return_values() {
        let rendered = render_to_value(&[
            Value::Str(Rc::new("<h1>Hello</h1>".into())),
            Value::Str(Rc::new("h1 { height: 2px }".into())),
            Value::Int(40),
        ])
        .unwrap();
        match rendered {
            Value::Str(text) => assert!(text.contains("<h1>")),
            other => panic!("expected str, got {other:?}"),
        }
    }
}
