use super::super::*;

#[path = "scroll_metrics/edges_parse.rs"]
mod parse;

pub(super) fn border(handle: &DomHandle) -> [i64; 4] {
    let Some(styled) = styled_node_at_path(handle) else {
        return [0; 4];
    };
    let mut edges = parse::box_values(styled.styles.get("border-width")).unwrap_or([0; 4]);
    edges[0] = edge(
        &styled.styles,
        "border-width-top",
        "border-top-width",
        edges[0],
    );
    edges[1] = edge(
        &styled.styles,
        "border-width-right",
        "border-right-width",
        edges[1],
    );
    edges[2] = edge(
        &styled.styles,
        "border-width-bottom",
        "border-bottom-width",
        edges[2],
    );
    edges[3] = edge(
        &styled.styles,
        "border-width-left",
        "border-left-width",
        edges[3],
    );
    edges
}

fn edge(styles: &HashMap<String, String>, legacy: &str, standard: &str, fallback: i64) -> i64 {
    styles
        .get(legacy)
        .and_then(|value| parse::part(value))
        .or_else(|| styles.get(standard).and_then(|value| parse::part(value)))
        .unwrap_or(fallback)
        .max(0)
}
