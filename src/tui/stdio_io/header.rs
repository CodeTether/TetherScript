//! Content-Length header parsing for stdio frames.

pub(super) fn content_length(line: &str) -> Result<Option<usize>, String> {
    let Some((name, value)) = clean(line).split_once(':') else {
        return Ok(None);
    };
    if !name.trim().eq_ignore_ascii_case("content-length") {
        return Ok(None);
    }
    value
        .trim()
        .parse::<usize>()
        .map(Some)
        .map_err(|error| format!("stdio_read: invalid Content-Length: {error}"))
}

pub(super) fn is_blank(line: &str) -> bool {
    clean(line).is_empty()
}

fn clean(line: &str) -> &str {
    line.trim_end_matches(['\r', '\n'])
        .trim_start_matches('\u{feff}')
}
