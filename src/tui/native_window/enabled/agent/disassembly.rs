//! Real executable text-section state for the CPU visualizer.

#[path = "disassembly_state.rs"]
mod state;

use object::{Object, ObjectSection, SectionKind};

pub(super) struct Snapshot {
    pub architecture: String,
    pub entry: u64,
    pub base: u64,
    pub size: u64,
    pub rows: Vec<super::disassembly_decode::Row>,
    pub selected: usize,
}
impl Snapshot {
    pub fn current() -> Result<Self, String> {
        let path = std::env::current_exe().map_err(|e| e.to_string())?;
        let bytes = std::fs::read(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
        let file = object::File::parse(bytes.as_slice()).map_err(|e| e.to_string())?;
        let section = file
            .sections()
            .find(|section| section.kind() == SectionKind::Text && section.size() > 0)
            .ok_or("executable has no text section")?;
        let base = section.address();
        let data = section.data().map_err(|e| e.to_string())?;
        Ok(Self {
            architecture: format!("{:?}", file.architecture()),
            entry: file.entry(),
            base,
            size: section.size(),
            rows: super::disassembly_decode::decode(data, base),
            selected: 0,
        })
    }
}
