pub fn inline_scripts(source: &str) -> Vec<&str> {
    let mut scripts = Vec::new();
    let mut rest = source;
    loop {
        let Some(open) = rest.find("<script") else {
            return scripts;
        };
        rest = &rest[open..];
        let Some(header_end) = rest.find('>') else {
            return scripts;
        };
        let header = &rest[..=header_end];
        let body = &rest[header_end + 1..];
        let Some(close) = body.find("</script>") else {
            return scripts;
        };
        if !header.contains("src=") {
            scripts.push(&body[..close]);
        }
        rest = &body[close + "</script>".len()..];
    }
}

#[cfg(test)]
mod tests {
    use super::inline_scripts;

    #[test]
    fn ignores_external_harness_scripts() {
        let html = "<script src='/h.js'></script><script>done();</script>";
        assert_eq!(inline_scripts(html), ["done();"]);
    }
}
