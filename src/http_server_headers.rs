//! Bounded HTTP request-header parsing.

use std::collections::HashMap;
use std::io::BufRead;
use std::rc::Rc;

use crate::value::Value;

use super::http_server_reader;

const MAX_LINE_BYTES: usize = 8 * 1024;
const MAX_HEADER_BYTES: usize = 64 * 1024;
const MAX_BODY_BYTES: usize = 1024 * 1024;

pub(super) fn read<R: BufRead>(reader: &mut R) -> Result<(HashMap<String, Value>, usize), String> {
    let mut headers = HashMap::new();
    let mut content_length = 0usize;
    let mut bytes = 0usize;
    while let Some(line) = http_server_reader::line(reader, MAX_LINE_BYTES, "header")? {
        bytes += line.len();
        if bytes > MAX_HEADER_BYTES {
            return Err(format!("headers exceed {MAX_HEADER_BYTES} bytes"));
        }
        let line = line.trim_end_matches(['\r', '\n']);
        if line.is_empty() {
            break;
        }
        if let Some((name, value)) = line.split_once(':') {
            let name = name.trim().to_ascii_lowercase();
            let value = value.trim().to_string();
            if name == "content-length" {
                content_length = value.parse().unwrap_or(0);
                if content_length > MAX_BODY_BYTES {
                    return Err(format!(
                        "content-length {content_length} exceeds {MAX_BODY_BYTES} bytes"
                    ));
                }
            }
            headers.insert(name, Value::Str(Rc::new(value)));
        }
    }
    Ok((headers, content_length))
}
