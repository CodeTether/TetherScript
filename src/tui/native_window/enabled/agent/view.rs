//! Agent dashboard projection into the in-tree HTML/CSS renderer.

use super::state::AgentState;

pub(super) fn document(state: &AgentState) -> (String, String) {
    let mut messages = String::new();
    for message in state.messages.iter().rev().take(18).rev() {
        messages.push_str(&format!(
            "<article><b>{}</b><p>{}</p></article>",
            escape(&message.role),
            escape(&message.text)
        ));
    }
    let html = format!(
        "<main><header><h1>tetherscript native agent</h1><span>{}</span></header>\
         <aside><b>MODE</b><p>{}</p><b>MODEL</b><p>{}</p><b>WORKSPACE</b><p>{}</p></aside>\
         <section>{}<footer><b>PROMPT</b><p>{}_</p><small>{}</small></footer></section></main>",
        escape(&state.status),
        escape(&state.mode),
        escape(&state.model),
        escape(&state.workspace),
        messages,
        escape(&state.input),
        if state.busy {
            "agent working"
        } else {
            "Enter sends, Escape closes"
        }
    );
    (html, super::view_style::CSS.into())
}

fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for character in text.chars() {
        match character {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(character),
        }
    }
    out
}
