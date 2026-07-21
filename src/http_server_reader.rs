//! Bounded HTTP/1.1 line reading.

use std::io::{BufRead, ErrorKind};

pub(super) fn line<R: BufRead>(
    reader: &mut R,
    limit: usize,
    label: &str,
) -> Result<Option<String>, String> {
    let mut out = Vec::new();
    loop {
        let available = reader.fill_buf().map_err(|error| {
            if matches!(error.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) {
                "empty request".to_string()
            } else {
                format!("read {label}: {error}")
            }
        })?;
        if available.is_empty() {
            if out.is_empty() {
                return Ok(None);
            }
            break;
        }
        let take = available
            .iter()
            .position(|byte| *byte == b'\n')
            .map_or(available.len(), |position| position + 1);
        if out.len() + take > limit {
            return Err(format!("{label} exceeds {limit} bytes"));
        }
        let complete = available[..take].last() == Some(&b'\n');
        out.extend_from_slice(&available[..take]);
        reader.consume(take);
        if complete {
            break;
        }
    }
    String::from_utf8(out)
        .map(Some)
        .map_err(|error| format!("read {label}: invalid UTF-8: {error}"))
}
