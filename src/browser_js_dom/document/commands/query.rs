use super::*;

pub(super) fn enabled(command: command::Command) -> bool {
    use command::Command::*;

    match command {
        Copy => !selection_host::command::text().is_empty(),
        Cut | Delete => {
            selection_host::command::focused_editable().is_some()
                && !selection_host::command::text().is_empty()
        }
        Paste => selection_host::command::focused_editable().is_some() && state::available(),
        InsertText | SelectAll => selection_host::command::focused_editable().is_some(),
    }
}
