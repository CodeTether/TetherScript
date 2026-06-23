use super::{body, chat_json};

fn parse(text: &str) -> crate::value::Value {
    crate::json::parse_str(text).unwrap()
}

#[test]
fn body_carries_tools_and_function_outputs() {
    let messages = parse(
        r#"[{"role":"user","content":"read file"},
        {"role":"assistant","content":"","tool_calls":[{"id":"call_1",
        "type":"function","function":{"name":"read","arguments":"{}"}}]},
        {"role":"tool","tool_call_id":"call_1","content":"body"}]"#,
    );
    let opts = parse(
        r#"{"model":"gpt-5","tools":[{"type":"function","function":{
        "name":"read","description":"Read file","parameters":{"type":"object"}}}]}"#,
    );
    let text = body::build(&[messages, opts], 0).unwrap();
    assert!(text.contains("\"tools\""));
    assert!(text.contains("\"function_call\""));
    assert!(text.contains("\"function_call_output\""));
    assert!(text.contains("\"call_1\""));
    assert!(text.contains("\"name\":\"read\""));
}

#[test]
fn sse_function_call_becomes_chat_tool_call() {
    let text = concat!(
        "data: {\"type\":\"response.output_item.done\",\"item\":{",
        "\"type\":\"function_call\",\"call_id\":\"call_2\",",
        "\"name\":\"cwd\",\"arguments\":\"{}\"}}\n\n",
        "data: [DONE]\n\n",
    );
    let out = chat_json(text).unwrap();
    let encoded = crate::json::encode_to_string(&out).unwrap();
    assert!(encoded.contains("\"tool_calls\""));
    assert!(encoded.contains("\"id\":\"call_2\""));
    assert!(encoded.contains("\"name\":\"cwd\""));
    assert!(encoded.contains("\"arguments\":\"{}\""));
}

#[test]
fn sse_completed_empty_output_keeps_stream_text() {
    let text = concat!(
        "data: {\"type\":\"response.output_text.delta\",\"delta\":\"Hello\"}\n\n",
        "data: {\"type\":\"response.completed\",\"response\":{\"output\":[]}}\n\n",
    );
    let out = chat_json(text).unwrap();
    let encoded = crate::json::encode_to_string(&out).unwrap();
    assert!(encoded.contains("\"content\":\"Hello\""));
}
