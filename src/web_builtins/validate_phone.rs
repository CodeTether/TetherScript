//! Phone number normalization to E.164 digits.
//!
//! The reference repo stores `phone_number` as free text, so the same subscriber
//! arrives as `(651) 555-0100`, `651-555-0100`, and `+1 651 555 0100`. Normalizing
//! to digits makes those compare equal, which is what dedupe and SMS delivery
//! need.

/// Strip formatting from a phone number, keeping only digits and a leading `+`.
///
/// # Arguments
///
/// * `text` — Candidate number in any common punctuation style.
///
/// # Returns
///
/// The digits, prefixed with `+` when the input carried one.
///
/// # Errors
///
/// Returns an error when the digit count falls outside the E.164 range of 7 to
/// 15, or when the input contains a character that is neither a digit nor
/// recognized punctuation. Unexpected characters are rejected rather than
/// silently dropped: quietly discarding a letter would turn a typo into a
/// different, valid-looking number.
pub(super) fn normalize_phone(text: &str) -> Result<String, String> {
    let trimmed = text.trim();
    // Only a leading `+` is meaningful; one later is punctuation, not a country
    // code, so it is rejected below rather than moved.
    let plus = trimmed.starts_with('+');
    let mut digits = String::new();

    for (index, ch) in trimmed.char_indices() {
        if ch.is_ascii_digit() {
            digits.push(ch);
        } else if ch == '+' && index == 0 {
            continue;
        } else if !matches!(ch, ' ' | '-' | '(' | ')' | '.' | '/') {
            return Err(format!(
                "normalize_phone: unexpected character `{ch}` at position {index}"
            ));
        }
    }

    if digits.len() < 7 {
        return Err(format!(
            "normalize_phone: {} digits is fewer than the E.164 minimum of 7",
            digits.len()
        ));
    }
    if digits.len() > 15 {
        return Err(format!(
            "normalize_phone: {} digits exceeds the E.164 maximum of 15",
            digits.len()
        ));
    }

    Ok(if plus { format!("+{digits}") } else { digits })
}
