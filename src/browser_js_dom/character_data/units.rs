pub(super) fn length(value: &str) -> usize {
    value.encode_utf16().count()
}

pub(super) fn slice(
    value: &str,
    offset: usize,
    count: usize,
    method: &str,
) -> Result<String, String> {
    let units = value.encode_utf16().collect::<Vec<_>>();
    if offset > units.len() {
        return Err(format!(
            "IndexSizeError: CharacterData.{method} offset {offset} exceeds length {}",
            units.len()
        ));
    }
    let end = offset.saturating_add(count).min(units.len());
    Ok(String::from_utf16_lossy(&units[offset..end]))
}

pub(super) fn replace(
    value: &str,
    offset: usize,
    count: usize,
    data: &str,
    method: &str,
) -> Result<String, String> {
    slice(value, offset, 0, method)?;
    let prefix = slice(value, 0, offset, method)?;
    let suffix_offset = offset.saturating_add(count).min(length(value));
    let suffix = slice(value, suffix_offset, usize::MAX, method)?;
    Ok(format!("{prefix}{data}{suffix}"))
}
