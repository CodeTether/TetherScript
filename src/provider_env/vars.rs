//! Environment variable helpers.

pub(super) fn get(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

pub(super) fn first(names: &[&str]) -> Option<String> {
    names.iter().find_map(|name| get(name))
}
