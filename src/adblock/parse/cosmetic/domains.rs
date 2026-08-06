//! Domain splitting for cosmetic filter declarations.

pub(super) fn split_domains(raw: &str) -> (Vec<String>, Vec<String>) {
    let mut doms = Vec::new();
    let mut excl = Vec::new();
    for part in raw.split(',') {
        let part = part.trim();
        if let Some(rest) = part.strip_prefix('~') {
            excl.push(rest.to_lowercase());
        } else if !part.is_empty() {
            doms.push(part.to_lowercase());
        }
    }
    (doms, excl)
}
