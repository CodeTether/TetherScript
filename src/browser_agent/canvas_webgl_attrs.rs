//! WebGL rendering-state attribute decoding.

use crate::browser::Element;

pub(super) fn u8(element: &Element, name: &str) -> Option<u8> {
    element.attrs.get(name).and_then(|value| value.parse().ok())
}

pub(super) fn u32(element: &Element, name: &str) -> Option<u32> {
    element.attrs.get(name).and_then(|value| value.parse().ok())
}

pub(super) fn boolean(element: &Element, name: &str) -> bool {
    element.attrs.get(name).is_some_and(|value| value == "true")
}

pub(super) fn array<T: Copy + std::str::FromStr>(
    element: &Element,
    name: &str,
    default: T,
) -> [T; 4] {
    let mut out = [default; 4];
    if let Some(raw) = element.attrs.get(name) {
        for (index, part) in raw.split(',').take(4).enumerate() {
            out[index] = part.parse().unwrap_or(default);
        }
    }
    out
}

pub(super) fn list(element: &Element, name: &str) -> Vec<String> {
    element
        .attrs
        .get(name)
        .map(|raw| {
            raw.split(';')
                .filter(|name| !name.is_empty())
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}
