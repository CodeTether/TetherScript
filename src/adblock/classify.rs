//! Classification helpers for uBlock/ABP options.

use super::ResourceType;

use self::apply::{collect_domains, set_type};

mod apply;

/// Parsed `$`-options modifiers.
pub(super) struct Options {
    pub third_party: Option<bool>,
    pub domains: Vec<String>,
    pub excluded_domains: Vec<String>,
    pub resource_types: ResourceType,
}

/// Split options into structured modifiers.
pub(super) fn parse_options(options: &str) -> Options {
    let mut opts = Options {
        third_party: None,
        domains: Vec::new(),
        excluded_domains: Vec::new(),
        resource_types: ResourceType::all(),
    };
    let any_type = false;
    for part in options.split(',') {
        classify(&mut opts, any_type, part.trim());
    }
    opts
}

fn classify(opts: &mut Options, any_type: bool, part: &str) {
    match part {
        "third-party" | "3p" => opts.third_party = Some(true),
        "~third-party" | "~3p" => opts.third_party = Some(false),
        "script" => set_type(opts, any_type, |r| r.script = true),
        "image" => set_type(opts, any_type, |r| r.image = true),
        "stylesheet" => set_type(opts, any_type, |r| r.stylesheet = true),
        "subdocument" => set_type(opts, any_type, |r| r.subdocument = true),
        "object" => set_type(opts, any_type, |r| r.object = true),
        "xhr" => set_type(opts, any_type, |r| r.xhr = true),
        "websocket" => set_type(opts, any_type, |r| r.websocket = true),
        "other" => set_type(opts, any_type, |r| r.other = true),
        _ if part.starts_with("domain=") => collect_domains(opts, &part[7..]),
        _ => {}
    }
}
