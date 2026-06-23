//! Script-declared authority headers.

const PREFIX: &str = "tetherscript:";

pub(crate) fn full_access(source: &str, cli_full_access: bool) -> bool {
    cli_full_access || header_requests_agent(source)
}

fn header_requests_agent(source: &str) -> bool {
    for line in source.lines().take(32) {
        let text = line.trim();
        if text.is_empty() {
            continue;
        }
        if !text.starts_with("//") {
            break;
        }
        if directive(text).is_some_and(agent_directive) {
            return true;
        }
    }
    false
}

fn directive(comment: &str) -> Option<&str> {
    comment
        .trim_start_matches('/')
        .trim()
        .strip_prefix(PREFIX)
        .map(str::trim)
}

fn agent_directive(value: &str) -> bool {
    matches!(
        value,
        "authority agent" | "agent authority" | "access-mode full"
    )
}
