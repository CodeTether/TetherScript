use super::*;

#[test]
fn parses_supported_commands_case_insensitively() {
    let copy = JsValue::String(" CoPy ".into());
    let bold = JsValue::String("bold".into());

    assert!(matches!(
        command::Command::parse(Some(&copy)),
        Some(command::Command::Copy)
    ));
    assert!(command::Command::parse(Some(&bold)).is_none());
    assert!(command::Command::parse(None).is_none());
}

#[test]
fn clipboard_state_distinguishes_empty_data_from_no_data() {
    state::reset();
    assert!(!state::available());
    state::write(String::new());
    assert!(state::available());
    assert_eq!(state::read(), Some(String::new()));
    state::reset();
    assert_eq!(state::read(), None);
}
