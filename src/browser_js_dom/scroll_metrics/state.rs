use super::*;

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub(super) struct Position {
    pub left: i64,
    pub top: i64,
}

thread_local! {
    static POSITIONS: RefCell<HashMap<String, Position>> = RefCell::new(HashMap::new());
}

pub(super) fn reset() {
    POSITIONS.with(|positions| positions.borrow_mut().clear());
}

pub(super) fn get(handle: &DomHandle) -> Position {
    POSITIONS.with(|positions| {
        positions
            .borrow()
            .get(&handle.event_key())
            .copied()
            .unwrap_or_default()
    })
}

pub(super) fn set(handle: &DomHandle, position: Position) {
    POSITIONS.with(|positions| {
        positions.borrow_mut().insert(handle.event_key(), position);
    });
}

pub(super) fn rekey(old: &str, new: &str) {
    POSITIONS.with(|positions| {
        let mut positions = positions.borrow_mut();
        if let Some(position) = positions.remove(old) {
            positions.insert(new.into(), position);
        }
    });
}

pub(super) fn ancestor_offset(handle: &DomHandle) -> (i64, i64) {
    let mut offset = Position::default();
    for depth in 1..handle.path.len() {
        let ancestor = DomHandle {
            root: handle.root.clone(),
            path: handle.path[..depth].to_vec(),
        };
        let position = get(&ancestor);
        offset.left = offset.left.saturating_add(position.left);
        offset.top = offset.top.saturating_add(position.top);
    }
    (offset.left, offset.top)
}
