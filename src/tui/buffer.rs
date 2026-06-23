//! Terminal frame buffer composition.

/// Collects shaped terminal rows into one frame string.
pub(super) struct Buffer {
    rows: Vec<String>,
}

impl Buffer {
    /// Create an empty row buffer.
    pub(super) fn new() -> Self {
        Self { rows: Vec::new() }
    }

    /// Append a complete terminal row without a trailing newline.
    pub(super) fn push(&mut self, row: String) {
        self.rows.push(row);
    }

    /// Convert rows to newline-terminated terminal frame text.
    pub(super) fn finish(self) -> String {
        let mut out = String::new();
        for row in self.rows {
            out.push_str(&row);
            out.push('\n');
        }
        out
    }
}
