//! HTML helpers for deterministic external script execution.

pub(crate) fn append_script(mut html: String, url: &str, source: &str) -> String {
    let marker = marker(url);
    if html.contains(&marker) {
        return html;
    }
    html.push_str("<script data-agent-resource-script=\"");
    html.push_str(&escape_attr(url));
    html.push_str("\">");
    html.push_str(source);
    html.push_str("</script>");
    html
}

fn marker(url: &str) -> String {
    format!("data-agent-resource-script=\"{}\"", escape_attr(url))
}

fn escape_attr(value: &str) -> String {
    value.replace('&', "&amp;").replace('"', "&quot;")
}
