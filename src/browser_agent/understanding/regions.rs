//! Page region detection.

use super::ElementSummary;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PageRegion {
    Header, Navigation, Main, Footer, Sidebar, Modal, Dialog, Banner, Complementary,
}

#[derive(Clone, Debug)]
pub struct RegionInfo { pub kind: PageRegion, pub selector: String, pub confidence: f64 }

fn has(s: &str, n: &str) -> bool { s.to_ascii_lowercase().contains(n) }

fn region_for(e: &ElementSummary) -> Option<(PageRegion, f64)> {
    let tag = e.tag.to_ascii_lowercase();
    let role = e.role.clone().unwrap_or_default().to_ascii_lowercase();
    let id = e.id.clone().unwrap_or_default();
    let class = e.classes.join(" ");
    let all = format!("{} {} {} {}", tag, role, id, class);
    if tag == "header" || role == "banner" { return Some((PageRegion::Header, 0.95)); }
    if tag == "nav" || role == "navigation" { return Some((PageRegion::Navigation, 0.95)); }
    if tag == "main" || role == "main" { return Some((PageRegion::Main, 0.95)); }
    if tag == "footer" || role == "contentinfo" { return Some((PageRegion::Footer, 0.95)); }
    if role == "dialog" { return Some((PageRegion::Dialog, 0.95)); }
    if has(&all, "modal") { return Some((PageRegion::Modal, 0.80)); }
    if has(&all, "sidebar") || has(&all, "aside") { return Some((PageRegion::Sidebar, 0.75)); }
    if tag == "aside" || role == "complementary" { return Some((PageRegion::Complementary, 0.85)); }
    None
}

pub fn detect_regions(elements: &[ElementSummary]) -> Vec<RegionInfo> {
    elements.iter().filter_map(|e| {
        region_for(e).map(|(kind, confidence)| RegionInfo {
            kind, selector: e.selector.clone(), confidence,
        })
    }).collect()
}
