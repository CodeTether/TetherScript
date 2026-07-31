//! SCRAM-SHA-256 client exchange (RFC 5802 / RFC 7677).
//!
//! Only the parts PostgreSQL uses are implemented: the client-first bare
//! message, parsing the server-first challenge, and producing the client-final
//! proof. Channel binding is not offered, so the GS2 header stays `n,,`.

use super::hmac::{hmac_sha256, pbkdf2_sha256};
use crate::system::{base64_decode_bytes, base64_encode_bytes, sha256};

pub(super) use super::nonce::client_nonce;

/// Parsed `r=`, `s=`, and `i=` attributes from the server-first message.
#[derive(Debug)]
pub(super) struct ServerFirst {
    pub(super) nonce: String,
    pub(super) salt: Vec<u8>,
    pub(super) iterations: u32,
}

/// Split `key=value` attribute pairs, returning the value for `key`.
fn attribute(message: &str, key: char) -> Option<&str> {
    message
        .split(',')
        .find(|part| part.starts_with(&format!("{key}=")))
        .map(|part| &part[2..])
}

pub(super) fn parse_server_first(message: &str) -> Result<ServerFirst, String> {
    let nonce = attribute(message, 'r').ok_or("postgres: SCRAM server-first is missing r=")?;
    let salt_b64 = attribute(message, 's').ok_or("postgres: SCRAM server-first is missing s=")?;
    let iterations = attribute(message, 'i')
        .ok_or("postgres: SCRAM server-first is missing the iteration count (i=)")?;
    Ok(ServerFirst {
        nonce: nonce.to_string(),
        salt: base64_decode_bytes(salt_b64)
            .map_err(|error| format!("postgres: SCRAM salt is not valid base64: {error}"))?,
        iterations: iterations.parse().map_err(|_| {
            format!("postgres: SCRAM iteration count `{iterations}` is not a number")
        })?,
    })
}

/// Build the client-final message, including the computed proof.
pub(super) fn client_final(
    password: &str,
    client_nonce: &str,
    first: &ServerFirst,
    client_first_bare: &str,
    server_first: &str,
) -> Result<String, String> {
    if !first.nonce.starts_with(client_nonce) {
        return Err("postgres: SCRAM server nonce does not extend the client nonce".into());
    }
    let salted = pbkdf2_sha256(password.as_bytes(), &first.salt, first.iterations);
    let client_key = hmac_sha256(&salted, b"Client Key");
    let stored_key = sha256(&client_key);

    // channel-binding-data is empty for `n,,`, whose base64 form is "biws".
    let without_proof = format!("c=biws,r={}", first.nonce);
    let auth_message = format!("{client_first_bare},{server_first},{without_proof}");
    let signature = hmac_sha256(&stored_key, auth_message.as_bytes());

    let mut proof = [0u8; 32];
    for (index, byte) in proof.iter_mut().enumerate() {
        *byte = client_key[index] ^ signature[index];
    }
    Ok(format!("{without_proof},p={}", base64_encode_bytes(&proof)))
}
