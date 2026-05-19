//! Form reset default action — snapshot initial values lazily.

use super::super::*;

type Initials = HashMap<String, HashMap<String, String>>;

thread_local! {
    static INITIAL_VALUES: RefCell<Initials> = RefCell::new(HashMap::new());
}

pub(super) fn reset() {
    INITIAL_VALUES.with(|vals| vals.borrow_mut().clear());
}

pub(super) fn perform(handle: &DomHandle) -> Result<(), String> {
    let key = handle_key(handle);
    ensure_snapshot(&key, handle);
    let initials = INITIAL_VALUES
        .with(|init| init.borrow().get(&key).cloned())
        .unwrap_or_default();
    restore_controls(handle, &initials)
}

fn ensure_snapshot(key: &str, handle: &DomHandle) {
    INITIAL_VALUES.with(|init| {
        if !init.borrow().contains_key(key) {
            let mut vals = HashMap::new();
            snapshot_controls(handle, &mut vals);
            init.borrow_mut().insert(key.to_string(), vals);
        }
    });
}

pub(super) fn ensure_snapshot_public(handle: &DomHandle) {
    ensure_snapshot(&handle_key(handle), handle);
}

fn restore_controls(handle: &DomHandle, initials: &HashMap<String, String>) -> Result<(), String> {
    let Some(node) = handle.node() else {
        return Ok(());
    };
    let Node::Element(el) = node else {
        return Ok(());
    };
    let pk = path_key(&handle.path);
    if el.tag == "input" || el.tag == "textarea" {
        if let Some(default) = initials.get(&pk) {
            handle.set_input_value(default.clone());
        }
    }
    if el.tag == "input" && is_checkable(&el) {
        let was_checked = initials
            .get(&format!("{}:checked", pk))
            .is_some_and(|v| v == "true");
        handle.set_checked_state(was_checked);
    }
    let child_count = el.children.len();
    for i in 0..child_count {
        let mut cp = handle.path.clone();
        cp.push(i);
        restore_controls(
            &DomHandle {
                root: handle.root.clone(),
                path: cp,
            },
            initials,
        )?;
    }
    Ok(())
}

fn snapshot_controls(handle: &DomHandle, vals: &mut HashMap<String, String>) {
    let Some(node) = handle.node() else { return };
    let Node::Element(el) = node else { return };
    let pk = path_key(&handle.path);
    if el.tag == "input" || el.tag == "textarea" {
        vals.insert(pk.clone(), handle.input_value());
    }
    if el.tag == "input" && is_checkable(&el) {
        vals.insert(
            format!("{}:checked", pk),
            el.attrs.contains_key("checked").to_string(),
        );
    }
    for (i, child) in el.children.iter().enumerate() {
        if let Node::Element(_) = child {
            let mut cp = handle.path.clone();
            cp.push(i);
            snapshot_controls(
                &DomHandle {
                    root: handle.root.clone(),
                    path: cp,
                },
                vals,
            );
        }
    }
}

fn is_checkable(el: &Element) -> bool {
    matches!(
        el.attrs.get("type").map(|t| t.as_str()),
        Some("checkbox" | "radio")
    )
}

fn path_key(path: &[usize]) -> String {
    path.iter()
        .map(|p| p.to_string())
        .collect::<Vec<_>>()
        .join(",")
}

fn handle_key(handle: &DomHandle) -> String {
    format!("{:p}", Rc::as_ptr(&handle.root))
}
