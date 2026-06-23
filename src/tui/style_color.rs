//! ANSI color name lookup for TUI styles.

pub(super) fn fg_code(name: &str) -> Option<&'static str> {
    code(name, false)
}

pub(super) fn bg_code(name: &str) -> Option<&'static str> {
    code(name, true)
}

fn code(name: &str, bg: bool) -> Option<&'static str> {
    let key = name.trim().to_ascii_lowercase().replace('-', "_");
    match (bg, key.as_str()) {
        (false, "black") => Some("30"),
        (false, "red") => Some("31"),
        (false, "green") => Some("32"),
        (false, "yellow") => Some("33"),
        (false, "blue") => Some("34"),
        (false, "magenta") => Some("35"),
        (false, "cyan") => Some("36"),
        (false, "white") => Some("37"),
        (false, "gray") | (false, "grey") | (false, "bright_black") => Some("90"),
        (false, "bright_red") => Some("91"),
        (false, "bright_green") => Some("92"),
        (false, "bright_yellow") => Some("93"),
        (false, "bright_blue") => Some("94"),
        (false, "bright_magenta") => Some("95"),
        (false, "bright_cyan") => Some("96"),
        (false, "bright_white") => Some("97"),
        (true, "black") => Some("40"),
        (true, "red") => Some("41"),
        (true, "green") => Some("42"),
        (true, "yellow") => Some("43"),
        (true, "blue") => Some("44"),
        (true, "magenta") => Some("45"),
        (true, "cyan") => Some("46"),
        (true, "white") => Some("47"),
        (true, "gray") | (true, "grey") | (true, "bright_black") => Some("100"),
        (true, "bright_red") => Some("101"),
        (true, "bright_green") => Some("102"),
        (true, "bright_yellow") => Some("103"),
        (true, "bright_blue") => Some("104"),
        (true, "bright_magenta") => Some("105"),
        (true, "bright_cyan") => Some("106"),
        (true, "bright_white") => Some("107"),
        _ => None,
    }
}
