pub fn requests(url: &str) -> String {
    format!(
        "{{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"tools/call\",\"params\":{{\"name\":\"browser_goto\",\"arguments\":{{\"url\":\"{}\"}}}}}}\n\
         {{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/call\",\"params\":{{\"name\":\"browser_click\",\"arguments\":{{\"selector\":\"#change\"}}}}}}\n\
         {{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{{\"name\":\"browser_text\",\"arguments\":{{\"selector\":\"#change\"}}}}}}\n\
         {{\"jsonrpc\":\"2.0\",\"id\":4,\"method\":\"tools/call\",\"params\":{{\"name\":\"browser_snapshot\",\"arguments\":{{}}}}}}\n",
        url
    )
}
