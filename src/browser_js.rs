//! Tiny dependency-free browser JavaScript host bindings.
//!
//! This is a deterministic bridge between the in-tree HTML/CSS browser
//! primitives and the in-tree JavaScript interpreter. It intentionally uses only
//! `std` and the project's own modules. It exposes a small DOM surface:
//! `document`, `window`, `getElementById`, `querySelector`, `querySelectorAll`,
//! `textContent`, `innerText`, `innerHTML`, `children`, `setAttribute`, and
//! `getAttribute`, plus basic element creation and tree mutation APIs.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::browser::{self, Element, Node};
use crate::js::{self, JsEngine, JsValue, NativeFunction};
use crate::value::Value;

const DOM_API_VERSION: &str = "tetherscript-dom-0.2";

#[derive(Clone, Copy)]
enum InsertPosition {
    Append,
    Prepend,
}

#[derive(Clone)]
struct DomHandle {
    root: Rc<RefCell<Node>>,
    path: Vec<usize>,
}

pub struct BrowserJsResult {
    pub document: browser::Document,
    pub value: JsValue,
    pub console: Vec<String>,
}

pub fn run_html_scripts(html: &str) -> Result<BrowserJsResult, String> {
    let root = html_to_root(html);
    let mut engine = JsEngine::new();
    install_dom_globals(&mut engine, root.clone());
    let scripts = collect_inline_scripts(&root.borrow());
    let mut last = JsValue::Undefined;
    for source in scripts {
        if !source.trim().is_empty() {
            last = engine.eval(&source)?;
        }
    }
    Ok(BrowserJsResult { document: root_to_document(&root), value: last, console: engine.console_output() })
}

pub fn eval_with_dom(html: &str, script: &str) -> Result<BrowserJsResult, String> {
    let root = html_to_root(html);
    let mut engine = JsEngine::new();
    install_dom_globals(&mut engine, root.clone());
    let value = engine.eval(script)?;
    Ok(BrowserJsResult { document: root_to_document(&root), value, console: engine.console_output() })
}

fn html_to_root(html: &str) -> Rc<RefCell<Node>> {
    let document = browser::parse_html(html);
    Rc::new(RefCell::new(Node::Element(Element {
        tag: "#document".into(),
        attrs: HashMap::new(),
        children: document.children,
    })))
}

fn install_dom_globals(engine: &mut JsEngine, root: Rc<RefCell<Node>>) {
    let document = node_object(DomHandle { root: root.clone(), path: Vec::new() });
    engine.set_global("document", document.clone());
    let mut window = HashMap::new();
    window.insert("document".into(), document);
    engine.set_global("window", JsValue::Object(Rc::new(RefCell::new(window))));
}

fn node_object(handle: DomHandle) -> JsValue {
    let node = handle.node().unwrap_or(Node::Text(String::new()));
    let mut obj = HashMap::new();
    obj.insert("nodeType".into(), JsValue::Number(if matches!(node, Node::Text(_)) { 3.0 } else if node_name(&node) == "#document" { 9.0 } else { 1.0 }));
    obj.insert("nodeName".into(), JsValue::String(node_name(&node)));
    obj.insert("tagName".into(), JsValue::String(node_name(&node).to_ascii_uppercase()));
    obj.insert("textContent".into(), JsValue::String(text_content_raw(&node)));
    obj.insert("innerText".into(), JsValue::String(browser::text_content(&node)));
    obj.insert("innerHTML".into(), JsValue::String(inner_html(&node)));
    obj.insert("children".into(), children_array(&handle, &node));
    obj.insert("childElementCount".into(), JsValue::Number(child_element_count(&node) as f64));

    if let Node::Element(el) = &node {
        obj.insert("id".into(), JsValue::String(el.attrs.get("id").cloned().unwrap_or_default()));
        obj.insert("className".into(), JsValue::String(el.attrs.get("class").cloned().unwrap_or_default()));
    }

    install_property_setters(&mut obj, &handle);

    let h = handle.clone();
    obj.insert("createElement".into(), native("createElement", Some(1), move |args| {
        let tag = args.first().unwrap_or(&JsValue::Undefined).display().to_ascii_lowercase();
        let path = h.append_child(Node::Element(Element { tag, attrs: HashMap::new(), children: Vec::new() }), InsertPosition::Append);
        Ok(node_object(DomHandle { root: h.root.clone(), path }))
    }));

    let h = handle.clone();
    obj.insert("createTextNode".into(), native("createTextNode", Some(1), move |args| {
        let text = args.first().unwrap_or(&JsValue::Undefined).display();
        let path = h.append_child(Node::Text(text), InsertPosition::Append);
        Ok(node_object(DomHandle { root: h.root.clone(), path }))
    }));

    let h = handle.clone();
    obj.insert("appendChild".into(), native("appendChild", Some(1), move |args| {
        let child = js_value_to_node(args.first().unwrap_or(&JsValue::Undefined));
        let path = h.append_child(child, InsertPosition::Append);
        Ok(node_object(DomHandle { root: h.root.clone(), path }))
    }));

    let h = handle.clone();
    obj.insert("getAttribute".into(), native("getAttribute", Some(1), move |args| {
        let name = args.first().unwrap_or(&JsValue::Undefined).display();
        Ok(match h.node() {
            Some(Node::Element(el)) => el.attrs.get(&name).map(|s| JsValue::String(s.clone())).unwrap_or(JsValue::Null),
            _ => JsValue::Null,
        })
    }));

    let h = handle.clone();
    obj.insert("setAttribute".into(), native("setAttribute", Some(2), move |args| {
        let name = args.first().unwrap_or(&JsValue::Undefined).display();
        let value = args.get(1).unwrap_or(&JsValue::Undefined).display();
        h.with_node_mut(|node| {
            if let Node::Element(el) = node { el.attrs.insert(name, value); }
        });
        Ok(JsValue::Undefined)
    }));

    let h = handle.clone();
    obj.insert("removeAttribute".into(), native("removeAttribute", Some(1), move |args| {
        let name = args.first().unwrap_or(&JsValue::Undefined).display();
        h.with_node_mut(|node| { if let Node::Element(el) = node { el.attrs.remove(&name); } });
        Ok(JsValue::Undefined)
    }));

    let h = handle.clone();
    obj.insert("hasAttribute".into(), native("hasAttribute", Some(1), move |args| {
        let name = args.first().unwrap_or(&JsValue::Undefined).display();
        Ok(JsValue::Bool(matches!(h.node(), Some(Node::Element(el)) if el.attrs.contains_key(&name))))
    }));

    let h = handle.clone();
    obj.insert("remove".into(), native("remove", Some(0), move |_| {
        h.remove_self();
        Ok(JsValue::Undefined)
    }));

    let h = handle.clone();
    obj.insert("prepend".into(), native("prepend", None, move |args| {
        for arg in args.iter().rev() { h.append_child(js_value_to_node(arg), InsertPosition::Prepend); }
        Ok(JsValue::Undefined)
    }));

    let h = handle.clone();
    obj.insert("append".into(), native("append", None, move |args| {
        for arg in args { h.append_child(js_value_to_node(arg), InsertPosition::Append); }
        Ok(JsValue::Undefined)
    }));

    let h = handle.clone();
    obj.insert("getElementById".into(), native("getElementById", Some(1), move |args| {
        let id = args.first().unwrap_or(&JsValue::Undefined).display();
        Ok(find_by_id(&h.root, &id).map(|path| node_object(DomHandle { root: h.root.clone(), path })).unwrap_or(JsValue::Null))
    }));

    let h = handle.clone();
    obj.insert("querySelector".into(), native("querySelector", Some(1), move |args| {
        let selector = args.first().unwrap_or(&JsValue::Undefined).display();
        Ok(find_by_selector(&h.root, &selector).map(|path| node_object(DomHandle { root: h.root.clone(), path })).unwrap_or(JsValue::Null))
    }));

    let h = handle;
    obj.insert("querySelectorAll".into(), native("querySelectorAll", Some(1), move |args| {
        let selector = args.first().unwrap_or(&JsValue::Undefined).display();
        let nodes = all_by_selector(&h.root, &selector).into_iter().map(|path| node_object(DomHandle { root: h.root.clone(), path })).collect();
        Ok(JsValue::Array(Rc::new(RefCell::new(nodes))))
    }));

    obj.insert("__domApiVersion".into(), JsValue::String(DOM_API_VERSION.into()));

    JsValue::Object(Rc::new(RefCell::new(obj)))
}

impl DomHandle {
    fn node(&self) -> Option<Node> { get_node(&self.root.borrow(), &self.path).cloned() }

    fn with_node_mut(&self, f: impl FnOnce(&mut Node)) {
        if let Some(node) = get_node_mut(&mut self.root.borrow_mut(), &self.path) { f(node); }
    }

    fn append_child(&self, child: Node, position: InsertPosition) -> Vec<usize> {
        let mut root = self.root.borrow_mut();
        let parent = get_node_mut(&mut root, &self.path);
        let Some(Node::Element(el)) = parent else { return self.path.clone(); };
        let index = match position {
            InsertPosition::Append => { el.children.push(child); el.children.len() - 1 }
            InsertPosition::Prepend => { el.children.insert(0, child); 0 }
        };
        let mut path = self.path.clone();
        path.push(index);
        path
    }

    fn remove_self(&self) -> bool {
        let Some((&index, parent_path)) = self.path.split_last() else { return false; };
        let mut root = self.root.borrow_mut();
        let Some(Node::Element(parent)) = get_node_mut(&mut root, parent_path) else { return false; };
        if index < parent.children.len() { parent.children.remove(index); true } else { false }
    }
}

fn install_property_setters(obj: &mut HashMap<String, JsValue>, handle: &DomHandle) {
    for prop in ["textContent", "innerText"] {
        let h = handle.clone();
        obj.insert(format!("__set:{}", prop), native(&format!("set_{}", prop), Some(1), move |args| {
            let text = args.first().unwrap_or(&JsValue::Undefined).display();
            h.with_node_mut(|node| match node {
                Node::Text(existing) => *existing = text,
                Node::Element(el) => el.children = vec![Node::Text(text)],
            });
            Ok(JsValue::Undefined)
        }));
    }

    let h = handle.clone();
    obj.insert("__set:id".into(), native("set_id", Some(1), move |args| {
        let value = args.first().unwrap_or(&JsValue::Undefined).display();
        h.with_node_mut(|node| { if let Node::Element(el) = node { el.attrs.insert("id".into(), value); } });
        Ok(JsValue::Undefined)
    }));

    let h = handle.clone();
    obj.insert("__set:className".into(), native("set_className", Some(1), move |args| {
        let value = args.first().unwrap_or(&JsValue::Undefined).display();
        h.with_node_mut(|node| { if let Node::Element(el) = node { el.attrs.insert("class".into(), value); } });
        Ok(JsValue::Undefined)
    }));
}

fn js_value_to_node(value: &JsValue) -> Node {
    match value {
        JsValue::Object(obj) => {
            let obj = obj.borrow();
            if let Some(JsValue::String(tag)) = obj.get("nodeName") {
                if tag != "#text" && tag != "#document" {
                    let mut attrs = HashMap::new();
                    if let Some(JsValue::String(id)) = obj.get("id") { if !id.is_empty() { attrs.insert("id".into(), id.clone()); } }
                    if let Some(JsValue::String(class_name)) = obj.get("className") { if !class_name.is_empty() { attrs.insert("class".into(), class_name.clone()); } }
                    let text = obj.get("textContent").map(JsValue::display).unwrap_or_default();
                    return Node::Element(Element {
                        tag: tag.to_ascii_lowercase(),
                        attrs,
                        children: if text.is_empty() { Vec::new() } else { vec![Node::Text(text)] },
                    });
                }
            }
            Node::Text(value.display())
        }
        _ => Node::Text(value.display()),
    }
}

fn get_node<'a>(node: &'a Node, path: &[usize]) -> Option<&'a Node> {
    if path.is_empty() { return Some(node); }
    match node {
        Node::Element(el) => el.children.get(path[0]).and_then(|child| get_node(child, &path[1..])),
        Node::Text(_) => None,
    }
}

fn get_node_mut<'a>(node: &'a mut Node, path: &[usize]) -> Option<&'a mut Node> {
    if path.is_empty() { return Some(node); }
    match node {
        Node::Element(el) => el.children.get_mut(path[0]).and_then(|child| get_node_mut(child, &path[1..])),
        Node::Text(_) => None,
    }
}

fn native(name: &str, arity: Option<usize>, func: impl Fn(&[JsValue]) -> Result<JsValue, String> + 'static) -> JsValue {
    JsValue::Native(Rc::new(NativeFunction::new(name, arity, func)))
}

fn root_to_document(root: &Rc<RefCell<Node>>) -> browser::Document {
    match &*root.borrow() {
        Node::Element(el) if el.tag == "#document" => browser::Document { children: el.children.clone() },
        node => browser::Document { children: vec![node.clone()] },
    }
}

fn collect_inline_scripts(node: &Node) -> Vec<String> {
    let mut out = Vec::new();
    collect_scripts(node, &mut out);
    out
}

fn collect_scripts(node: &Node, out: &mut Vec<String>) {
    if let Node::Element(el) = node {
        if el.tag.eq_ignore_ascii_case("script") && !el.attrs.contains_key("src") {
            out.push(el.children.iter().map(text_content_raw).collect::<Vec<_>>().join(""));
        }
        for child in &el.children { collect_scripts(child, out); }
    }
}

fn find_by_id(root: &Rc<RefCell<Node>>, id: &str) -> Option<Vec<usize>> {
    find_path(&root.borrow(), &mut Vec::new(), &|node| matches!(node, Node::Element(el) if el.attrs.get("id").map(|s| s.as_str()) == Some(id)))
}

fn find_by_selector(root: &Rc<RefCell<Node>>, selector: &str) -> Option<Vec<usize>> {
    find_path(&root.borrow(), &mut Vec::new(), &|node| node_matches_simple_selector(node, selector))
}

fn all_by_selector(root: &Rc<RefCell<Node>>, selector: &str) -> Vec<Vec<usize>> {
    let mut out = Vec::new();
    collect_paths(&root.borrow(), &mut Vec::new(), &|node| node_matches_simple_selector(node, selector), &mut out);
    out
}

fn find_path(node: &Node, path: &mut Vec<usize>, pred: &impl Fn(&Node) -> bool) -> Option<Vec<usize>> {
    if pred(node) { return Some(path.clone()); }
    if let Node::Element(el) = node {
        for (index, child) in el.children.iter().enumerate() {
            path.push(index);
            if let Some(found) = find_path(child, path, pred) { return Some(found); }
            path.pop();
        }
    }
    None
}

fn collect_paths(node: &Node, path: &mut Vec<usize>, pred: &impl Fn(&Node) -> bool, out: &mut Vec<Vec<usize>>) {
    if pred(node) { out.push(path.clone()); }
    if let Node::Element(el) = node {
        for (index, child) in el.children.iter().enumerate() {
            path.push(index);
            collect_paths(child, path, pred, out);
            path.pop();
        }
    }
}

fn node_matches_simple_selector(node: &Node, selector: &str) -> bool {
    let Node::Element(el) = node else { return false; };
    let selector = selector.trim();
    if selector.is_empty() { return false; }
    if let Some(id) = selector.strip_prefix('#') { return el.attrs.get("id").map(|s| s == id).unwrap_or(false); }
    if let Some(class) = selector.strip_prefix('.') {
        return el.attrs.get("class").map(|s| s.split_whitespace().any(|c| c == class)).unwrap_or(false);
    }
    el.tag.eq_ignore_ascii_case(selector)
}

fn node_name(node: &Node) -> String {
    match node { Node::Element(el) => el.tag.clone(), Node::Text(_) => "#text".into() }
}

fn children_array(handle: &DomHandle, node: &Node) -> JsValue {
    let len = match node { Node::Element(el) => el.children.len(), Node::Text(_) => 0 };
    let children = (0..len).map(|index| {
        let mut path = handle.path.clone();
        path.push(index);
        node_object(DomHandle { root: handle.root.clone(), path })
    }).collect();
    JsValue::Array(Rc::new(RefCell::new(children)))
}

fn text_content_raw(node: &Node) -> String {
    match node {
        Node::Text(text) => text.clone(),
        Node::Element(el) => el.children.iter().map(text_content_raw).collect::<Vec<_>>().join(""),
    }
}

fn inner_html(node: &Node) -> String {
    match node {
        Node::Text(text) => escape_html(text),
        Node::Element(el) => el.children.iter().map(outer_html).collect::<Vec<_>>().join(""),
    }
}

fn outer_html(node: &Node) -> String {
    match node {
        Node::Text(text) => escape_html(text),
        Node::Element(el) => {
            if el.tag == "#document" { return inner_html(node); }
            let mut out = String::new();
            out.push('<'); out.push_str(&el.tag);
            for (k, v) in &el.attrs { out.push(' '); out.push_str(k); out.push_str("=\""); out.push_str(&escape_attr(v)); out.push('"'); }
            out.push('>'); out.push_str(&inner_html(node)); out.push_str("</"); out.push_str(&el.tag); out.push('>');
            out
        }
    }
}

fn escape_html(text: &str) -> String { text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;") }
fn escape_attr(text: &str) -> String { escape_html(text).replace('"', "&quot;") }

pub fn browser_run_scripts_to_value(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 { return Err(format!("browser_run_scripts: expected 1 arg, got {}", args.len())); }
    let Value::Str(html) = &args[0] else { return Err(format!("browser_run_scripts: expected str, got {}", args[0].type_name())); };
    result_to_value(run_html_scripts(html)?)
}

pub fn browser_eval_js_to_value(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 { return Err(format!("browser_eval_js: expected 2 args, got {}", args.len())); }
    let Value::Str(html) = &args[0] else { return Err(format!("browser_eval_js: html must be str, got {}", args[0].type_name())); };
    let Value::Str(script) = &args[1] else { return Err(format!("browser_eval_js: script must be str, got {}", args[1].type_name())); };
    result_to_value(eval_with_dom(html, script)?)
}

fn result_to_value(result: BrowserJsResult) -> Result<Value, String> {
    let mut map = HashMap::new();
    map.insert("dom".into(), document_value(&result.document));
    map.insert("value".into(), js::js_to_tether(&result.value));
    map.insert("console".into(), Value::List(Rc::new(RefCell::new(result.console.into_iter().map(|s| Value::Str(Rc::new(s))).collect()))));
    Ok(Value::Map(Rc::new(RefCell::new(map))))
}

fn document_value(document: &browser::Document) -> Value {
    Value::List(Rc::new(RefCell::new(document.children.iter().map(node_value).collect())))
}

fn node_value(node: &Node) -> Value {
    let mut map = HashMap::new();
    match node {
        Node::Text(text) => { map.insert("type".into(), Value::Str(Rc::new("text".into()))); map.insert("text".into(), Value::Str(Rc::new(text.clone()))); }
        Node::Element(el) => {
            map.insert("type".into(), Value::Str(Rc::new("element".into())));
            map.insert("tag".into(), Value::Str(Rc::new(el.tag.clone())));
            map.insert("attrs".into(), Value::Map(Rc::new(RefCell::new(el.attrs.iter().map(|(k, v)| (k.clone(), Value::Str(Rc::new(v.clone())))).collect()))));
            map.insert("children".into(), Value::List(Rc::new(RefCell::new(el.children.iter().map(node_value).collect()))));
        }
    }
    Value::Map(Rc::new(RefCell::new(map)))
}

fn child_element_count(node: &Node) -> usize {
    match node {
        Node::Element(el) => el.children.iter().filter(|child| matches!(child, Node::Element(_))).count(),
        Node::Text(_) => 0,
    }
}

pub fn compatibility_report_to_value(_args: &[Value]) -> Result<Value, String> {
    let features = ["document", "window", "selectors", "attributes", "textContent", "innerHTML", "createElement", "append", "remove"];
    Ok(Value::List(Rc::new(RefCell::new(features.into_iter().map(|s| Value::Str(Rc::new(s.into()))).collect()))))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inline_scripts_can_read_document_and_console_log() {
        let html = "<main id='app'><h1>Hello</h1><script>console.log(document.getElementById('app').children.length); document.querySelector('h1').textContent;</script></main>";
        let result = run_html_scripts(html).unwrap();
        assert_eq!(result.value, JsValue::String("Hello".into()));
        assert_eq!(result.console, vec!["2".to_string()]);
    }

    #[test]
    fn eval_with_dom_exposes_selectors_and_attributes() {
        let result = eval_with_dom("<p class='note' id='x'>Hi</p>", "let p=document.querySelector('.note'); p.setAttribute('data-ok','yes'); p.getAttribute('id') + ':' + p.textContent;").unwrap();
        assert_eq!(result.value, JsValue::String("x:Hi".into()));
        match &result.document.children[0] {
            Node::Element(el) => assert_eq!(el.attrs.get("data-ok"), Some(&"yes".to_string())),
            Node::Text(_) => panic!("expected element"),
        }
    }

    #[test]
    fn dom_property_assignment_and_mutation_apis_update_document() {
        let result = eval_with_dom(
            "<main id='app'><p>old</p></main>",
            "let app=document.getElementById('app'); let p=document.querySelector('p'); p.textContent='new'; let span=document.createElement('span'); span.textContent='!'; app.appendChild(span); document.getElementById('app').children.length;",
        ).unwrap();
        assert_eq!(result.value, JsValue::Number(2.0));
        let text = result.document.children.iter().map(browser::text_content).collect::<Vec<_>>().join(" ");
        assert!(text.contains("new"));
        assert!(text.contains("!"));
    }
}