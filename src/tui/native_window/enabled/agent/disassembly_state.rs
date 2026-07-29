//! Disassembly selection and unavailable-state helpers.

use super::Snapshot;

impl Snapshot {
    pub fn placeholder(error: String) -> Self {
        Self {
            architecture: error,
            entry: 0,
            base: 0,
            size: 0,
            rows: Vec::new(),
            selected: 0,
        }
    }

    pub fn selected_address(&self) -> u64 {
        self.rows
            .get(self.selected)
            .map_or(self.entry, |row| row.address)
    }
}
