//! Text-node `CharacterData` properties and mutation methods.

use super::*;

#[path = "character_data/args.rs"]
mod args;
#[path = "character_data/methods.rs"]
mod methods;
#[path = "character_data/ops.rs"]
mod ops;
#[path = "character_data/props.rs"]
mod props;
#[path = "character_data/state.rs"]
mod state;
#[path = "character_data/units.rs"]
mod units;

pub(super) fn install(obj: &mut HashMap<String, JsValue>, handle: &DomHandle, node: &Node) {
    if matches!(node, Node::Text(_)) {
        props::install(obj, handle);
        methods::install(obj, handle);
    }
}

#[cfg(test)]
#[path = "character_data/tests.rs"]
mod tests;
