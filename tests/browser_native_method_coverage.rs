use std::collections::BTreeSet;

#[test]
fn native_e2e_calls_every_exposed_browser_method() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let dispatch = std::fs::read_to_string(root.join("src/browser_cap/map_dispatch.rs")).unwrap();
    let example = std::fs::read_to_string(root.join("examples/native_browser_e2e.tether")).unwrap();
    let mut methods = quoted_names(&dispatch);
    methods.extend(
        "describe trace export_trace_json export_har agent_summary minimal_reproduction_script"
            .split_whitespace()
            .map(str::to_string),
    );
    for method in methods {
        let call = if method.contains('.') {
            format!("browser.\"{method}\"(")
        } else {
            format!("browser.{method}(")
        };
        assert!(example.contains(&call), "native E2E does not call {call}");
    }
}

fn quoted_names(source: &str) -> BTreeSet<String> {
    source
        .split('"')
        .skip(1)
        .step_by(2)
        .filter(|value| {
            !value.is_empty()
                && value
                    .chars()
                    .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || "_.".contains(ch))
        })
        .map(str::to_string)
        .collect()
}
