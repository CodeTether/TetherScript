use super::*;

pub(super) fn read(handle: &DomHandle) -> String {
    match handle.node() {
        Some(Node::Text(value)) => value,
        _ => String::new(),
    }
}

pub(super) fn write(handle: &DomHandle, old: String, next: String) {
    handle.with_node_mut(|node| {
        if let Node::Text(value) = node {
            *value = next;
        }
    });
    queue_mutation_record(
        handle,
        "characterData",
        None,
        Some(old),
        Vec::new(),
        Vec::new(),
    );
}
