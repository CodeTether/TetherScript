//! Ownership-count validation and transfer diagnostics.

pub(super) fn aggregate(
    count: usize,
    operation: &str,
    kind: Option<&'static str>,
    owner: &str,
) -> Result<Option<&'static str>, String> {
    if let Some(kind) = kind {
        check(count, operation, kind, owner)?;
    }
    Ok(kind)
}

pub(super) fn check(count: usize, operation: &str, kind: &str, owner: &str) -> Result<(), String> {
    if count == 1 {
        return Ok(());
    }
    let subject = match owner {
        "resource" => format!("{kind} resource"),
        _ => format!("{owner} containing {kind} resource"),
    };
    Err(format!(
        "{operation}: cannot retain borrowed {subject}; use `move` to transfer ownership"
    ))
}
