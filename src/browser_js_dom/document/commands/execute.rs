use super::*;

pub(super) fn run(
    document: &DomHandle,
    command: command::Command,
    value: &str,
) -> Result<bool, String> {
    if !query::enabled(command) {
        return Ok(false);
    }
    use command::Command::*;
    match command {
        Copy => clipboard::copy(document),
        Cut => clipboard::cut(document),
        Paste => clipboard::paste(document),
        Delete => edit::replace(""),
        InsertText => edit::replace(value),
        SelectAll => edit::select_all(document),
    }
}
