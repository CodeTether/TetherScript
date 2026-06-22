//! Vault HTTP request writer.

use std::io::Write;

use super::url::Url;

pub(super) fn write(stream: &mut dyn Write, url: &Url, token: &str) -> Result<(), String> {
    write!(
        stream,
        "GET {} HTTP/1.1\r\n\
         Host: {}\r\n\
         User-Agent: tetherscript-vault/0.1\r\n\
         Accept: application/json\r\n\
         X-Vault-Token: {}\r\n\
         Connection: close\r\n\r\n",
        url.path,
        host_header(url),
        token
    )
    .map_err(|error| format!("vault: write request failed: {error}"))?;
    stream
        .flush()
        .map_err(|error| format!("vault: flush request failed: {error}"))
}

fn host_header(url: &Url) -> String {
    match (url.port, url.path.as_str()) {
        (80 | 443, _) => url.host.clone(),
        _ => format!("{}:{}", url.host, url.port),
    }
}
