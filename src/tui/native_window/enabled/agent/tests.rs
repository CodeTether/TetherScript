//! Native agent protocol and disassembly regression tests.

use super::{disassembly::Snapshot, model::Event, protocol};

#[test]
fn request_encodes_agent_prompt_as_json_rpc() {
    let encoded = protocol::request(7, "agent/message", Some("hello \"agent\""))
        .expect("request should encode");
    assert!(encoded.contains("\"id\":7"));
    assert!(encoded.contains("hello \\\"agent\\\""));
}

#[test]
fn state_response_restores_shared_agent_metadata() {
    let json = r#"{"jsonrpc":"2.0","id":1,"result":{"messages":[],"model":"glm-5","provider_ready":true,"session_path":".tetherscript/agent_tui_session.jsonl","workspace":"C:/repo"}}"#;
    let Some(Event::State {
        model,
        workspace,
        session,
        ready,
        ..
    }) = protocol::decode(json)
    else {
        panic!("state response should decode");
    };
    assert_eq!(model, "glm-5");
    assert_eq!(workspace, "C:/repo");
    assert!(session.ends_with("agent_tui_session.jsonl"));
    assert!(ready);
}

#[test]
fn current_executable_produces_real_disassembly_rows() {
    let snapshot = Snapshot::current().expect("test executable should have text section");
    assert!(!snapshot.rows.is_empty());
    assert!(!snapshot.rows[0].bytes.is_empty());
}
