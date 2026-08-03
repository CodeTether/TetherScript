//! PEM armour stripping and base64 decoding.
//!
//! A PEM document is `-----BEGIN <LABEL>-----`, a base64 body, then
//! `-----END <LABEL>-----`. This module removes the armour, checks that the two
//! labels agree, and hands the body to the existing in-tree base64 decoder,
//! `crate::system::base64_decode_bytes` — no second base64 implementation is
//! introduced here. That decoder already skips CR, LF, space, and tab, so
//! wrapped bodies work as-is, and a trailing newline after the END line is
//! optional.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::asn1::pem;
//!
//! // The DER bytes 30 03 02 01 05 base64-encode to "MAMCAQU=".
//! let armoured = "-----BEGIN PUBLIC KEY-----\nMAMCAQU=\n-----END PUBLIC KEY-----\n";
//! let block = pem::decode(armoured).unwrap();
//! assert_eq!(block.label, "PUBLIC KEY");
//! assert_eq!(block.der, vec![0x30, 0x03, 0x02, 0x01, 0x05]);
//!
//! // The trailing newline is optional.
//! let bare = armoured.trim_end();
//! assert_eq!(pem::decode(bare).unwrap().der, block.der);
//! ```

use super::error::Error;

/// A decoded PEM block: its label and the DER bytes it carried.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    /// The armour label, e.g. `"PUBLIC KEY"` or `"CERTIFICATE"`.
    pub label: String,
    /// The base64-decoded DER bytes.
    pub der: Vec<u8>,
}

/// Decode the first PEM block in `input`.
///
/// # Arguments
///
/// * `input` — text containing one PEM block, with or without a trailing
///   newline and with any line wrapping.
///
/// # Returns
///
/// The [`Block`] with its label and decoded DER bytes.
///
/// # Errors
///
/// [`Error::Pem`] when the BEGIN or END line is missing, the labels disagree, or
/// the base64 body is invalid. The offset is a byte offset into `input`.
///
/// # Panics
///
/// Never; all slicing goes through `str::find` results and `str::get`, and the
/// base64 step returns `Result`.
pub fn decode(input: &str) -> Result<Block, Error> {
    let armour = super::pem_armour::locate(input)?;
    let body: String = armour.body.chars().filter(|c| !c.is_whitespace()).collect();
    let der = crate::system::base64_decode_bytes(&body).map_err(|reason| Error::Pem {
        offset: armour.body_offset,
        reason,
    })?;
    Ok(Block {
        label: armour.label,
        der,
    })
}
