//! The `oauth_pkce_pair` result shape.
//!
//! Its own file so map construction is not tangled with either the crypto in
//! [`super::gen`] or the registration list in [`super::super::install_pkce`].
//!
//! `code_challenge_method` is returned alongside the pair rather than left for the
//! caller to remember. A caller that hardcodes the method string can hardcode
//! `"plain"` by mistake; one that copies this field cannot.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use super::METHOD;
use super::gen::{challenge, generate};
use crate::value::Value;

/// Mint a verifier and its S256 challenge.
///
/// # Returns
///
/// `Ok` of a map with `code_verifier` (43 characters), `code_challenge` (43
/// characters), and `code_challenge_method` (`"S256"`).
///
/// # Errors
///
/// Returns `Err` only if the freshly generated verifier fails its own validation,
/// which would mean a bug in [`generate`] rather than bad input. It is surfaced
/// rather than unwrapped so a broken entropy source can never yield a silently
/// weak verifier.
pub(crate) fn build() -> Result<Value, String> {
    let verifier = generate();
    let derived = challenge(&verifier)?;
    let mut map = HashMap::with_capacity(3);
    map.insert("code_verifier".to_string(), Value::Str(Rc::new(verifier)));
    map.insert("code_challenge".to_string(), Value::Str(Rc::new(derived)));
    map.insert(
        "code_challenge_method".to_string(),
        Value::Str(Rc::new(METHOD.to_string())),
    );
    Ok(Value::Map(Rc::new(RefCell::new(map))))
}
