//! Resource kind labels for network diagnostics.

use super::super::ResourceKind;

pub(super) fn name(kind: ResourceKind) -> &'static str {
    match kind {
        ResourceKind::Script => "script",
        ResourceKind::Stylesheet => "stylesheet",
        ResourceKind::Image => "image",
        ResourceKind::SourceMap => "source map",
    }
}
