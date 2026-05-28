//! Deterministic memory-pressure simulation for tests and diagnostics.

use crate::browser_agent::page::BrowserPage;

/// Result of a deterministic memory-pressure pass.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemoryPressureStatus {
    /// Number of non-essential caches dropped by the pass.
    pub dropped_non_essential_caches: usize,
    /// Whether the essential-cache marker remained present after trimming.
    pub essential_caches_retained: bool,
}

impl BrowserPage {
    /// Simulate memory pressure by dropping page-local non-essential caches.
    pub fn simulate_memory_pressure(&mut self) -> Result<MemoryPressureStatus, String> {
        let dropped = self
            .eval_js("let dropped=0;if(typeof window.__agentNonEssentialCache!='undefined'){delete window.__agentNonEssentialCache;dropped=1;}dropped")?
            .display()
            .parse::<usize>()
            .unwrap_or(0);
        let retained = self
            .eval_js("typeof window.__agentEssentialCache!='undefined'")?
            .display()
            == "true";
        Ok(MemoryPressureStatus {
            dropped_non_essential_caches: dropped,
            essential_caches_retained: retained,
        })
    }
}
