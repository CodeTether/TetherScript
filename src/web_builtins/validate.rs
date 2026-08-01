//! Input validation built-ins.
//!
//! The port accepts form and JSON input but had no way to check it, so every
//! handler would hand-roll the same predicates. The reference repo validates the
//! same four shapes throughout: `email` (304 references), `slug` (162),
//! `phone_number` (46), and `postal_code` (45).
//!
//! # Script surface
//!
//! | Builtin | Returns |
//! |---|---|
//! | `is_email(text)` | bool |
//! | `is_slug(text)` | bool |
//! | `is_digits(text)` | bool |
//! | `normalize_phone(text)` | `Result` of E.164 digits |
//! | `validate_fields(values, rules)` | `Result` of a field-to-message map |
//!
//! # What these do NOT claim
//!
//! `is_email` is a **pragmatic filter, not proof of deliverability**, and it is
//! not the RFC 5322 grammar. It rejects legal-but-exotic forms — quoted local
//! parts, comments, bracketed address literals — because accepting them buys
//! nothing for a signup form. An address that passes may still bounce: the only
//! way to know a mailbox exists is to send to it. Do not present a pass as
//! verification.
//!
//! There is no regex engine in the default build, so every check is a
//! hand-written single-pass scanner. That also removes any
//! catastrophic-backtracking risk on untrusted input.
//!
//! # Examples
//!
//! ```tether
//! let values = form_parse("email=a@b.com&slug=my-post").unwrap()
//!
//! let email_rules = map()
//! email_rules.required = true
//! email_rules.email = true
//!
//! let rules = map()
//! rules.email = email_rules
//!
//! let errors = validate_fields(values, rules).unwrap()
//! if errors.len() == 0 { println("input is valid") }
//! ```

use std::cell::RefCell;
use std::rc::Rc;

use crate::value::Env;

#[path = "validate_fields.rs"]
pub(super) mod validate_fields;
#[path = "validate_install.rs"]
pub(super) mod validate_install;
#[path = "validate_length.rs"]
pub(super) mod validate_length;
#[path = "validate_phone.rs"]
pub(super) mod validate_phone;
#[path = "validate_rule.rs"]
pub(super) mod validate_rule;
#[path = "validate_scan.rs"]
pub(super) mod validate_scan;

/// Register this group's built-ins.
pub(crate) fn install(env: &Rc<RefCell<Env>>) {
    validate_install::install(env);
}
