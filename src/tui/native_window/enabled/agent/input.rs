//! Native keyboard editing for the agent prompt.

use minifb::Key;

pub(super) fn apply(keys: &[Key], text: &[char], input: &mut String) -> bool {
    input.extend(text);
    let mut submit = false;
    for key in keys {
        match key {
            Key::Enter => submit = true,
            Key::Backspace => {
                input.pop();
            }
            _ => {}
        }
    }
    submit
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unicode_text_and_submit_are_preserved() {
        let mut input = String::new();
        assert!(apply(&[Key::Enter], &['H', 'é'], &mut input));
        assert_eq!(input, "Hé");
    }

    #[test]
    fn backspace_removes_one_unicode_scalar() {
        let mut input = "Hé".to_string();
        assert!(!apply(&[Key::Backspace], &[], &mut input));
        assert_eq!(input, "H");
    }
}
