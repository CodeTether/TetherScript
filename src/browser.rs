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
    pub selector_text: String,
    pub specificity: u32,
    pub order: usize,
    pub declarations: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Selector {
    pub parts: Vec<SimpleSelector>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SimpleSelector {
    pub tag: Option<String>,
    pub id: Option<String>,
    pub classes: Vec<String>,
    pub attrs: Vec<AttributeSelector>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AttributeSelector {
    pub name: String,
    pub value: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DisplayCommand {
    Rect {
        x: i64,
        y: i64,
        width: i64,
        height: i64,
        color: String,
    },
    Text {
        x: i64,
        y: i64,
        text: String,
        color: String,
    },
    Image {
        x: i64,
        y: i64,
        width: i64,
        height: i64,
        src: String,
    },
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
    let mut order = 0;
    for part in source.split('}') {
        let Some((selector_raw, body)) = part.split_once('{') else {
            continue;
        };
        let declarations = parse_declarations(body);
        if declarations.is_empty() {
            continue;
        }
        for selector_raw in selector_raw.split(',') {
            let selector_text = selector_raw.trim();
            if selector_text.is_empty() {
                continue;
            }
            let Some(selector) = parse_selector(selector_text) else {
                continue;
            };
            rules.push(CssRule {
                specificity: selector_specificity(&selector),
                selector,
                selector_text: selector_text.to_string(),
                order,
                declarations: declarations.clone(),
            });
            order += 1;
        }
    }
    rules
}

pub fn parse_inline_style(source: &str) -> HashMap<String, String> {
    parse_declarations(source)
}

pub fn computed_styles(document: &Document, css: &str) -> Vec<StyledNode> {
    let rules = parse_css(css);
    style_document(document, &rules)
}

pub fn query_selector(document: &Document, selector: &str) -> Vec<Node> {
    let Some(selector) = parse_selector(selector) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for child in &document.children {
        collect_matches(child, &[], &selector, &mut out);
    }
    out
}

pub fn element_matches(element: &Element, ancestors: &[Element], selector: &str) -> bool {
    let Some(selector) = parse_selector(selector) else {
        return false;
    };
    selector_matches(&selector, element, ancestors)
}

pub fn text_content(node: &Node) -> String {
    let mut out = String::new();
    collect_text(node, &mut out);
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn extract_embedded_css(document: &Document) -> String {
    let mut css = String::new();
    for child in &document.children {
        collect_style_text(child, &mut css);
    }
    css
}

pub fn page_snapshot(document: &Document, external_css: &str, width: i64) -> Value {
    let embedded_css = extract_embedded_css(document);
    let css = if external_css.trim().is_empty() {
        embedded_css.clone()
    } else if embedded_css.trim().is_empty() {
        external_css.to_string()
    } else {
        format!("{}\n{}", embedded_css, external_css)
    };
    let layout = layout_document(document, &css, width);
    let mut map = HashMap::new();
    map.insert("dom".into(), document_to_value(document));
    map.insert("css".into(), Value::Str(Rc::new(css)));
    map.insert("embedded_css".into(), Value::Str(Rc::new(embedded_css)));
    map.insert(
        "stylesheets".into(),
        string_list_to_value(&collect_attr_values(document, "link", "href")),
    );
    map.insert(
        "scripts".into(),
        string_list_to_value(&collect_attr_values(document, "script", "src")),
    );
    map.insert(
        "images".into(),
        string_list_to_value(&collect_attr_values(document, "img", "src")),
    );
    map.insert(
        "app_roots".into(),
        string_list_to_value(&detect_app_roots(document)),
    );
    map.insert("layout".into(), layout_to_value(&layout));
    map.insert(
        "display_list".into(),
        display_list_to_value(&build_display_list(&layout)),
    );
    map.insert("text".into(), Value::Str(Rc::new(render_text(&layout))));
    Value::Map(Rc::new(RefCell::new(map)))
}

fn parse_declarations(source: &str) -> HashMap<String, String> {
    let mut declarations = HashMap::new();
    for decl in source.split(';') {
        let Some((name, value)) = decl.split_once(':') else {
            continue;
        };
        let name = name.trim().to_ascii_lowercase();
        let value = value.trim().to_string();
        if !name.is_empty() && !value.is_empty() {
            declarations.insert(name, value);
        }
    }
    declarations
}

pub fn style_document(document: &Document, rules: &[CssRule]) -> Vec<StyledNode> {
    document
        .children
        .iter()
        .map(|node| style_node(node, &[], &HashMap::new(), rules))
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

pub fn build_display_list(layout: &LayoutBox) -> Vec<DisplayCommand> {
    let mut commands = Vec::new();
    collect_display_commands(layout, &mut commands);
    commands
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

pub fn display_list_to_value(commands: &[DisplayCommand]) -> Value {
    let values = commands
        .iter()
        .map(|cmd| {
            let mut map = HashMap::new();
            match cmd {
                DisplayCommand::Rect {
                    x,
                    y,
                    width,
                    height,
                    color,
                } => {
                    map.insert("type".into(), Value::Str(Rc::new("rect".into())));
                    map.insert("x".into(), Value::Int(*x));
                    map.insert("y".into(), Value::Int(*y));
                    map.insert("width".into(), Value::Int(*width));
                    map.insert("height".into(), Value::Int(*height));
                    map.insert("color".into(), Value::Str(Rc::new(color.clone())));
                }
                DisplayCommand::Text { x, y, text, color } => {
                    map.insert("type".into(), Value::Str(Rc::new("text".into())));
                    map.insert("x".into(), Value::Int(*x));
                    map.insert("y".into(), Value::Int(*y));
                    map.insert("text".into(), Value::Str(Rc::new(text.clone())));
                    map.insert("color".into(), Value::Str(Rc::new(color.clone())));
                }
                DisplayCommand::Image {
                    x,
                    y,
                    width,
                    height,
                    src,
                } => {
                    map.insert("type".into(), Value::Str(Rc::new("image".into())));
                    map.insert("x".into(), Value::Int(*x));
                    map.insert("y".into(), Value::Int(*y));
                    map.insert("width".into(), Value::Int(*width));
                    map.insert("height".into(), Value::Int(*height));
                    map.insert("src".into(), Value::Str(Rc::new(src.clone())));
                }
            }
            Value::Map(Rc::new(RefCell::new(map)))
        })
        .collect();
    Value::List(Rc::new(RefCell::new(values)))
}

fn style_node(
    node: &Node,
    ancestors: &[Element],
    inherited: &HashMap<String, String>,
    rules: &[CssRule],
) -> StyledNode {
    let mut styles = inherited_styles(inherited);
    let mut winning: HashMap<String, (u32, usize)> = HashMap::new();
    if let Node::Element(element) = node {
        for rule in rules {
            if selector_matches(&rule.selector, element, ancestors) {
                for (name, value) in &rule.declarations {
                    let key = (rule.specificity, rule.order);
                    if winning.get(name).is_none_or(|existing| key >= *existing) {
                        styles.insert(name.clone(), value.clone());
                        winning.insert(name.clone(), key);
                    }
                }
            }
        }
        if let Some(inline) = element.attrs.get("style") {
            for (name, value) in parse_inline_style(inline) {
                styles.insert(name, value);
            }
        }
        // Carry relevant DOM attributes into styles so the layout/display pipeline can
        // reference them (e.g. <img src="..."> produces a non-empty DisplayCommand::Image.src).
        if element.tag.eq_ignore_ascii_case("img") {
            if let Some(src) = element.attrs.get("src") {
                styles.entry("src".into()).or_insert_with(|| src.clone());
            }
        }
    }
    let children = match node {
        Node::Element(element) => {
            let mut next_ancestors = ancestors.to_vec();
            next_ancestors.push(element.clone());
            element
                .children
                .iter()
                .map(|child| style_node(child, &next_ancestors, &styles, rules))
                .collect()
        }
        Node::Text(_) => Vec::new(),
    };
    StyledNode {
        node: node.clone(),
        styles,
        children,
    }
}

fn inherited_styles(styles: &HashMap<String, String>) -> HashMap<String, String> {
    let mut inherited = HashMap::new();
    for key in ["color", "font-size", "font-family"] {
        if let Some(value) = styles.get(key) {
            inherited.insert(key.to_string(), value.clone());
        }
    }
    inherited
}

pub fn styled_node_to_value(styled: &StyledNode) -> Value {
    let mut map = HashMap::new();
    match &styled.node {
        Node::Text(text) => {
            map.insert("type".into(), Value::Str(Rc::new("text".into())));
            map.insert("text".into(), Value::Str(Rc::new(text.clone())));
        }
        Node::Element(element) => {
            map.insert("type".into(), Value::Str(Rc::new("element".into())));
            map.insert("tag".into(), Value::Str(Rc::new(element.tag.clone())));
            if let Some(id) = element.attrs.get("id") {
                map.insert("id".into(), Value::Str(Rc::new(id.clone())));
            }
            if let Some(classes) = element.attrs.get("class") {
                map.insert("class".into(), Value::Str(Rc::new(classes.clone())));
            }
            map.insert("attrs".into(), string_map_to_value(&element.attrs));
        }
    }
    map.insert("styles".into(), string_map_to_value(&styled.styles));
    let children = styled.children.iter().map(styled_node_to_value).collect();
    map.insert(
        "children".into(),
        Value::List(Rc::new(RefCell::new(children))),
    );
    Value::Map(Rc::new(RefCell::new(map)))
}

pub fn css_rules_to_value(rules: &[CssRule]) -> Value {
    let values = rules
        .iter()
        .map(|rule| {
            let mut map = HashMap::new();
            map.insert(
                "selector".into(),
                Value::Str(Rc::new(rule.selector_text.clone())),
            );
            map.insert("specificity".into(), Value::Int(rule.specificity as i64));
            map.insert(
                "declarations".into(),
                string_map_to_value(&rule.declarations),
            );
            Value::Map(Rc::new(RefCell::new(map)))
        })
        .collect();
    Value::List(Rc::new(RefCell::new(values)))
}

fn parse_selector(source: &str) -> Option<Selector> {
    let mut parts = Vec::new();
    for raw in source.split_whitespace() {
        let raw = raw.trim();
        if raw.is_empty() || raw.contains('>') || raw.contains('+') || raw.contains('~') {
            return None;
        }
        let part = parse_simple_selector(raw)?;
        parts.push(part);
    }
    if parts.is_empty() {
        None
    } else {
        Some(Selector { parts })
    }
}

fn parse_simple_selector(raw: &str) -> Option<SimpleSelector> {
    if raw == "*" {
        return Some(SimpleSelector {
            tag: None,
            id: None,
            classes: Vec::new(),
            attrs: Vec::new(),
        });
    }
    let mut tag = None;
    let mut id = None;
    let mut classes = Vec::new();
    let mut attrs = Vec::new();
    let mut current = String::new();
    let mut mode = 't';

    let chars: Vec<char> = raw.chars().collect();
    let mut index = 0;
    while index <= chars.len() {
        let ch = chars.get(index).copied().unwrap_or('\0');
        if ch == '#' || ch == '.' {
            flush_simple_selector_piece(&mut tag, &mut id, &mut classes, mode, &mut current);
            mode = ch;
            index += 1;
            continue;
        }
        if ch == '[' {
            flush_simple_selector_piece(&mut tag, &mut id, &mut classes, mode, &mut current);
            let mut attr = String::new();
            index += 1;
            while let Some(attr_ch) = chars.get(index).copied() {
                if attr_ch == ']' {
                    break;
                }
                attr.push(attr_ch);
                index += 1;
            }
            if chars.get(index) != Some(&']') {
                return None;
            }
            attrs.push(parse_attribute_selector(&attr)?);
            index += 1;
            continue;
        }
        if ch == '\0' {
            flush_simple_selector_piece(&mut tag, &mut id, &mut classes, mode, &mut current);
            break;
        }
        if ch == '*' && mode == 't' && current.is_empty() {
            index += 1;
            continue;
        }
        if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' || ch == ':' {
            current.push(ch);
        } else {
            return None;
        }
        index += 1;
    }

    Some(SimpleSelector {
        tag,
        id,
        classes,
        attrs,
    })
}

fn flush_simple_selector_piece(
    tag: &mut Option<String>,
    id: &mut Option<String>,
    classes: &mut Vec<String>,
    mode: char,
    current: &mut String,
) {
    if current.is_empty() {
        return;
    }
    match mode {
        't' => {
            *tag = Some(current.to_ascii_lowercase());
            current.clear();
        }
        '#' => *id = Some(std::mem::take(current)),
        '.' => classes.push(std::mem::take(current)),
        _ => current.clear(),
    }
}

fn parse_attribute_selector(raw: &str) -> Option<AttributeSelector> {
    let raw = raw.trim();
    if raw.is_empty() {
        return None;
    }
    let (name, value) = raw.split_once('=').map_or((raw, None), |(name, value)| {
        (
            name.trim(),
            Some(value.trim().trim_matches('"').trim_matches('\'')),
        )
    });
    if name.is_empty() {
        return None;
    }
    Some(AttributeSelector {
        name: name.to_ascii_lowercase(),
        value: value.map(str::to_string),
    })
}

fn selector_specificity(selector: &Selector) -> u32 {
    selector
        .parts
        .iter()
        .map(|part| {
            let ids = u32::from(part.id.is_some());
            let classes = part.classes.len() as u32;
            let tags = u32::from(part.tag.is_some());
            let attrs = part.attrs.len() as u32;
            ids * 100 + (classes + attrs) * 10 + tags
        })
        .sum()
}

fn selector_matches(selector: &Selector, element: &Element, ancestors: &[Element]) -> bool {
    let Some(last) = selector.parts.last() else {
        return false;
    };
    if !simple_selector_matches(last, element) {
        return false;
    }
    if selector.parts.len() == 1 {
        return true;
    }

    let mut ancestor_pos = ancestors.len();
    for part in selector.parts[..selector.parts.len() - 1].iter().rev() {
        let mut found = false;
        while ancestor_pos > 0 {
            ancestor_pos -= 1;
            if simple_selector_matches(part, &ancestors[ancestor_pos]) {
                found = true;
                break;
            }
        }
        if !found {
            return false;
        }
    }
    true
}

fn simple_selector_matches(selector: &SimpleSelector, element: &Element) -> bool {
    if selector.tag.as_ref().is_some_and(|tag| tag != &element.tag) {
        return false;
    }
    if selector
        .id
        .as_ref()
        .is_some_and(|id| element.attrs.get("id") != Some(id))
    {
        return false;
    }
    selector.classes.iter().all(|class| {
        element
            .attrs
            .get("class")
            .is_some_and(|classes| classes.split_whitespace().any(|item| item == class))
    }) && selector.attrs.iter().all(|attr| match &attr.value {
        Some(value) => element.attrs.get(&attr.name) == Some(value),
        None => element.attrs.contains_key(&attr.name),
    })
}

fn collect_matches(node: &Node, ancestors: &[Element], selector: &Selector, out: &mut Vec<Node>) {
    if let Node::Element(element) = node {
        if selector_matches(selector, element, ancestors) {
            out.push(node.clone());
        }
        let mut next_ancestors = ancestors.to_vec();
        next_ancestors.push(element.clone());
        for child in &element.children {
            collect_matches(child, &next_ancestors, selector, out);
        }
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
            let explicit_height = parse_px(styled.styles.get("height")).map(|height| height.max(0));
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

fn collect_display_commands(layout: &LayoutBox, out: &mut Vec<DisplayCommand>) {
    if layout.kind == "block" {
        if let Some(color) = layout
            .styles
            .get("background")
            .or_else(|| layout.styles.get("background-color"))
        {
            out.push(DisplayCommand::Rect {
                x: layout.x,
                y: layout.y,
                width: layout.width,
                height: layout.height,
                color: color.clone(),
            });
        }
        if layout.tag.as_deref() == Some("img") {
            let src = layout.styles.get("src").cloned().unwrap_or_default();
            out.push(DisplayCommand::Image {
                x: layout.x,
                y: layout.y,
                width: layout.width,
                height: layout.height,
                src,
            });
        }
    } else if layout.kind == "text" {
        if let Some(text) = &layout.text {
            if !text.trim().is_empty() {
                out.push(DisplayCommand::Text {
                    x: layout.x,
                    y: layout.y,
                    text: text.trim().to_string(),
                    color: layout
                        .styles
                        .get("color")
                        .cloned()
                        .unwrap_or_else(|| "black".into()),
                });
            }
        }
    }
    for child in &layout.children {
        collect_display_commands(child, out);
    }
}

fn collect_text(node: &Node, out: &mut String) {
    match node {
        Node::Text(text) => {
            out.push_str(text);
            out.push(' ');
        }
        Node::Element(element) => {
            for child in &element.children {
                collect_text(child, out);
            }
        }
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
            if let Some(id) = element.attrs.get("id") {
                map.insert("id".into(), Value::Str(Rc::new(id.clone())));
            }
            if let Some(classes) = element.attrs.get("class") {
                map.insert("class".into(), Value::Str(Rc::new(classes.clone())));
            }
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

fn string_list_to_value(input: &[String]) -> Value {
    Value::List(Rc::new(RefCell::new(
        input
            .iter()
            .map(|item| Value::Str(Rc::new(item.clone())))
            .collect(),
    )))
}

fn collect_style_text(node: &Node, out: &mut String) {
    if let Node::Element(element) = node {
        if element.tag == "style" {
            for child in &element.children {
                if let Node::Text(text) = child {
                    if !out.is_empty() {
                        out.push('\n');
                    }
                    out.push_str(text);
                }
            }
        }
        for child in &element.children {
            collect_style_text(child, out);
        }
    }
}

fn collect_attr_values(document: &Document, tag: &str, attr: &str) -> Vec<String> {
    let mut out = Vec::new();
    for child in &document.children {
        collect_attr_values_from_node(child, tag, attr, &mut out);
    }
    out
}

fn collect_attr_values_from_node(node: &Node, tag: &str, attr: &str, out: &mut Vec<String>) {
    if let Node::Element(element) = node {
        if element.tag == tag {
            if let Some(value) = element.attrs.get(attr) {
                out.push(value.clone());
            }
        }
        for child in &element.children {
            collect_attr_values_from_node(child, tag, attr, out);
        }
    }
}

fn detect_app_roots(document: &Document) -> Vec<String> {
    let mut out = Vec::new();
    for child in &document.children {
        detect_app_roots_from_node(child, &mut out);
    }
    out
}

fn detect_app_roots_from_node(node: &Node, out: &mut Vec<String>) {
    if let Node::Element(element) = node {
        let id = element
            .attrs
            .get("id")
            .map(String::as_str)
            .unwrap_or_default();
        let classes = element
            .attrs
            .get("class")
            .map(String::as_str)
            .unwrap_or_default();
        let framework_attr = element.attrs.keys().any(|key| {
            key.starts_with("ng-")
                || key.starts_with("data-ng-")
                || key.starts_with("v-")
                || key.starts_with("data-react")
        });
        let appish_name = matches!(
            id,
            "app" | "root" | "__next" | "angular" | "ng-app" | "react-root"
        ) || classes
            .split_whitespace()
            .any(|class| matches!(class, "app" | "root" | "ng-app" | "react-root" | "vue-app"));
        if appish_name || framework_attr {
            let label = if id.is_empty() {
                format!("{}[class='{}']", element.tag, classes)
            } else {
                format!("{}#{}", element.tag, id)
            };
            out.push(label);
        }
        for child in &element.children {
            detect_app_roots_from_node(child, out);
        }
    }
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
    let mut decoded = input.to_string();
    loop {
        let next = decoded
            .replace("&amp;", "&")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&quot;", "\"")
            .replace("&#39;", "'");
        if next == decoded {
            return decoded;
        }
        decoded = next;
    }
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

pub fn css_to_value(args: &[Value]) -> Result<Value, String> {
    let source = expect_str(args.first(), "browser_parse_css")?;
    Ok(css_rules_to_value(&parse_css(source)))
}

pub fn styles_to_value(args: &[Value]) -> Result<Value, String> {
    if !(1..=2).contains(&args.len()) {
        return Err(format!(
            "browser_styles: expected 1 to 2 args, got {}",
            args.len()
        ));
    }
    let html = expect_str(args.first(), "browser_styles")?;
    let css = match args.get(1) {
        Some(Value::Str(css)) => css.as_str(),
        Some(other) => {
            return Err(format!(
                "browser_styles: css must be str, got {}",
                other.type_name()
            ))
        }
        None => "",
    };
    let doc = parse_html(html);
    let styled = computed_styles(&doc, css)
        .iter()
        .map(styled_node_to_value)
        .collect();
    Ok(Value::List(Rc::new(RefCell::new(styled))))
}

pub fn query_selector_to_value(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!(
            "browser_query_selector: expected 2 args, got {}",
            args.len()
        ));
    }
    let html = expect_str(args.first(), "browser_query_selector")?;
    let selector = expect_str(args.get(1), "browser_query_selector")?;
    let doc = parse_html(html);
    let nodes = query_selector(&doc, selector)
        .iter()
        .map(node_to_value)
        .collect();
    Ok(Value::List(Rc::new(RefCell::new(nodes))))
}

pub fn text_content_to_value(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!(
            "browser_text_content: expected 2 args, got {}",
            args.len()
        ));
    }
    let html = expect_str(args.first(), "browser_text_content")?;
    let selector = expect_str(args.get(1), "browser_text_content")?;
    let doc = parse_html(html);
    let text = query_selector(&doc, selector)
        .first()
        .map(text_content)
        .unwrap_or_default();
    Ok(Value::Str(Rc::new(text)))
}

pub fn snapshot_to_value(args: &[Value]) -> Result<Value, String> {
    if !(1..=3).contains(&args.len()) {
        return Err(format!(
            "browser_snapshot: expected 1 to 3 args, got {}",
            args.len()
        ));
    }
    let html = expect_str(args.first(), "browser_snapshot")?;
    let css = match args.get(1) {
        Some(Value::Str(css)) => css.as_str(),
        Some(other) => {
            return Err(format!(
                "browser_snapshot: css must be str, got {}",
                other.type_name()
            ))
        }
        None => "",
    };
    let width = optional_width(args.get(2), "browser_snapshot")?;
    Ok(page_snapshot(&parse_html(html), css, width))
}

pub fn display_list_to_runtime_value(args: &[Value]) -> Result<Value, String> {
    if !(1..=3).contains(&args.len()) {
        return Err(format!(
            "browser_display_list: expected 1 to 3 args, got {}",
            args.len()
        ));
    }
    let html = expect_str(args.first(), "browser_display_list")?;
    let css = match args.get(1) {
        Some(Value::Str(css)) => css.as_str(),
        Some(other) => {
            return Err(format!(
                "browser_display_list: css must be str, got {}",
                other.type_name()
            ))
        }
        None => "",
    };
    let width = optional_width(args.get(2), "browser_display_list")?;
    let doc = parse_html(html);
    let layout = layout_document(&doc, css, width);
    Ok(display_list_to_value(&build_display_list(&layout)))
}

pub fn render_to_value(args: &[Value]) -> Result<Value, String> {
    if !(1..=3).contains(&args.len()) {
        return Err(format!(
            "browser_render: expected 1 to 3 args, got {}",
            args.len()
        ));
    }
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
    let width = optional_width(args.get(2), "browser_render")?;
    let doc = parse_html(html);
    let layout = layout_document(&doc, css, width);
    Ok(Value::Str(Rc::new(render_text(&layout))))
}

pub fn layout_to_runtime_value(args: &[Value]) -> Result<Value, String> {
    if !(1..=3).contains(&args.len()) {
        return Err(format!(
            "browser_layout: expected 1 to 3 args, got {}",
            args.len()
        ));
    }
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
    let width = optional_width(args.get(2), "browser_layout")?;
    let doc = parse_html(html);
    Ok(layout_to_value(&layout_document(&doc, css, width)))
}

fn optional_width(value: Option<&Value>, name: &str) -> Result<i64, String> {
    match value {
        Some(Value::Int(width)) => Ok(*width),
        Some(other) => Err(format!(
            "{}: width must be int, got {}",
            name,
            other.type_name()
        )),
        None => Ok(80),
    }
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

    #[test]
    fn decodes_nested_entities() {
        let doc = parse_html("<p>&amp;lt;tag&amp;gt;</p>");
        let Node::Element(element) = &doc.children[0] else {
            panic!("expected element");
        };
        assert_eq!(element.children, vec![Node::Text("<tag>".into())]);
    }

    #[test]
    fn clamps_negative_explicit_height() {
        let doc = parse_html("<section><p>Hello</p></section>");
        let layout = layout_document(&doc, "section { height: -5px }", 80);

        assert_eq!(layout.children[0].height, 0);
        assert_eq!(layout.height, 1);
    }

    #[test]
    fn css_supports_compound_descendant_and_inline_cascade() {
        let doc = parse_html(
            r#"<main id="app"><p class="note strong" style="height: 4px">Hello</p><p class="note">World</p></main>"#,
        );
        let styled = computed_styles(
            &doc,
            "p { height: 1px } main .note { color: blue } #app p.strong { height: 2px; color: red }",
        );
        let Node::Element(main) = &styled[0].node else {
            panic!("expected main");
        };
        assert_eq!(main.tag, "main");
        assert_eq!(
            styled[0].children[0].styles.get("color"),
            Some(&"red".into())
        );
        assert_eq!(
            styled[0].children[0].styles.get("height"),
            Some(&"4px".into())
        );
        assert_eq!(
            styled[0].children[1].styles.get("color"),
            Some(&"blue".into())
        );
    }

    #[test]
    fn query_selector_finds_descendant_matches() {
        let doc = parse_html(
            r#"<main id="app"><p class="note">One</p><section><p>Two</p></section></main>"#,
        );
        assert_eq!(query_selector(&doc, "#app p").len(), 2);
        assert_eq!(query_selector(&doc, "main .note").len(), 1);
    }

    #[test]
    fn query_selector_supports_attributes_and_text_content() {
        let doc = parse_html(
            r#"<div id="root" data-reactroot><button aria-label="Save"> Save <span>now</span></button></div>"#,
        );
        assert_eq!(query_selector(&doc, "[data-reactroot]").len(), 1);
        let buttons = query_selector(&doc, "button[aria-label='Save']");
        assert_eq!(buttons.len(), 1);
        assert_eq!(text_content(&buttons[0]), "Save now");
    }

    #[test]
    fn snapshot_extracts_framework_resources_and_embedded_css() {
        let doc = parse_html(
            r#"<html><head><link href="/app.css"><style>#root { background: green }</style><script src="/bundle.js"></script></head><body><div id="root"><img src="/logo.png"></div></body></html>"#,
        );
        let snapshot = page_snapshot(&doc, "", 80);
        let Value::Map(map) = snapshot else {
            panic!("expected snapshot map");
        };
        assert!(map.borrow().contains_key("display_list"));
        assert!(map
            .borrow()
            .get("css")
            .unwrap()
            .to_string()
            .contains("#root"));
        assert!(detect_app_roots(&doc).contains(&"div#root".to_string()));
    }

    #[test]
    fn display_list_contains_background_and_text_commands() {
        let doc = parse_html(r#"<div class="card"><p>Hello</p></div>"#);
        let layout = layout_document(&doc, ".card { background: red } p { color: blue }", 80);
        let commands = build_display_list(&layout);
        assert!(commands
            .iter()
            .any(|cmd| matches!(cmd, DisplayCommand::Rect { color, .. } if color == "red")));
        assert!(commands.iter().any(|cmd| matches!(cmd, DisplayCommand::Text { text, color, .. } if text == "Hello" && color == "blue")));
    }

    #[test]
    fn img_src_attribute_carried_into_display_command() {
        let doc = parse_html(r#"<img src="photo.png">"#);
        let layout = layout_document(&doc, "", 80);
        let commands = build_display_list(&layout);
        let image_cmd = commands.iter().find(|cmd| matches!(cmd, DisplayCommand::Image { .. }));
        assert!(image_cmd.is_some(), "expected an Image display command for <img>");
        if let Some(DisplayCommand::Image { src, .. }) = image_cmd {
            assert_eq!(src, "photo.png", "img src should come from DOM attribute, not styles");
        }
    }

    #[test]
    fn browser_variadics_reject_extra_args() {
        let args = [
            Value::Str(Rc::new("<h1>Hello</h1>".into())),
            Value::Str(Rc::new("".into())),
            Value::Int(80),
            Value::Int(1),
        ];

        assert!(render_to_value(&args)
            .unwrap_err()
            .contains("expected 1 to 3 args"));
        assert!(layout_to_runtime_value(&args)
            .unwrap_err()
            .contains("expected 1 to 3 args"));
    }
}
