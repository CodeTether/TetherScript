//! MutationObserver registry with filtering and delivery.

use super::mutation_types::{MutationObserverConfig, MutationRecord, MutationType};

pub type MutationCallback = Box<dyn FnMut(Vec<MutationRecord>)>;

pub struct MutationObserver {
    pub id: u64,
    pub target_node_id: u64,
    pub config: MutationObserverConfig,
    pub pending: Vec<MutationRecord>,
    callback: MutationCallback,
}

#[derive(Default)]
pub struct MutationObserverRegistry { next_id: u64, observers: Vec<MutationObserver> }

impl MutationObserverRegistry {
    pub fn new() -> Self { Self { next_id: 1, observers: vec![] } }
    pub fn observe(&mut self, target: u64, config: MutationObserverConfig, cb: MutationCallback) -> u64 {
        let id = self.next_id; self.next_id += 1;
        self.observers.push(MutationObserver { id, target_node_id: target, config, pending: vec![], callback: cb });
        id
    }
    pub fn disconnect(&mut self, id: u64) { self.observers.retain(|o| o.id != id); }
    pub fn take_records(&mut self, id: u64) -> Vec<MutationRecord> {
        self.observers.iter_mut().find(|o| o.id == id)
            .map(|o| std::mem::take(&mut o.pending)).unwrap_or_default()
    }
    pub fn queue_record(&mut self, record: MutationRecord, ancestors: &[u64]) {
        for obs in &mut self.observers {
            if obs.matches(&record, ancestors) { obs.pending.push(record.clone()); }
        }
    }
    pub fn deliver(&mut self) {
        for obs in &mut self.observers {
            let records = std::mem::take(&mut obs.pending);
            if !records.is_empty() { (obs.callback)(records); }
        }
    }
}

impl MutationObserver {
    fn matches(&self, record: &MutationRecord, ancestors: &[u64]) -> bool {
        if record.target_node_id != self.target_node_id
            && !(self.config.subtree && ancestors.contains(&self.target_node_id)) { return false; }
        match record.type_ {
            MutationType::ChildList => self.config.child_list,
            MutationType::Attributes => self.config.attributes
                && (self.config.attribute_filter.is_empty()
                    || record.attribute_name.as_ref().map_or(false, |n| self.config.attribute_filter.contains(n))),
            MutationType::CharacterData => self.config.character_data,
        }
    }
}
