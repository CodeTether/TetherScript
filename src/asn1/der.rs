//! The DER reader: a cursor over `&[u8]` that yields
//! [`Tlv`](super::tlv::Tlv) values.
//!
//! # Design
//!
//! Traversal is *iterative*, not recursive. Entering a SEQUENCE hands back a new
//! [`Reader`] borrowing the parent's content slice, so descending costs one
//! stack frame in the caller's own loop, never a recursive call inside the
//! decoder. A `depth` counter travels with each child reader and refuses to
//! exceed [`MAX_DEPTH`], so a document made of ten thousand nested SEQUENCEs
//! fails with `Error::DepthExceeded` instead of exhausting the stack.
//!
//! # Safety properties
//!
//! * Every public entry point returns `Result`; none of them can panic.
//! * Content octets are sliced in exactly one place and always via
//!   `slice::get(..)`, so a length claiming 4 GiB yields
//!   `Error::LengthExceedsInput` rather than a panic or an allocation.
//! * Indefinite lengths and non-minimal lengths are rejected.
//! * All offset arithmetic uses `saturating_add`, so no index can wrap.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::asn1::der::Reader;
//!
//! // SEQUENCE { INTEGER 5, BOOLEAN true }
//! let der = [0x30, 0x06, 0x02, 0x01, 0x05, 0x01, 0x01, 0xff];
//! let mut top = Reader::new(&der);
//! let mut seq = top.read_sequence().unwrap();
//! assert_eq!(seq.read_integer_bytes().unwrap(), &[0x05]);
//! assert!(seq.read_bool().unwrap());
//! assert!(seq.is_empty());
//! ```

/// Maximum nesting depth accepted by [`Reader`].
///
/// The top-level document is depth 0, so 32 levels of constructed nesting are
/// allowed. Real X.509 and PKCS#1 structures nest fewer than ten deep; the limit
/// exists purely to bound attacker-controlled work.
pub const MAX_DEPTH: usize = 32;

/// A cursor over DER-encoded bytes.
#[derive(Debug, Clone)]
pub struct Reader<'a> {
    pub(super) input: &'a [u8],
    pub(super) pos: usize,
    pub(super) base: usize,
    pub(super) depth: usize,
}

impl<'a> Reader<'a> {
    /// Create a reader over a whole DER document.
    ///
    /// # Arguments
    ///
    /// * `input` — the encoded bytes; may be empty.
    ///
    /// # Returns
    ///
    /// A reader positioned at offset 0 with depth 0.
    pub fn new(input: &'a [u8]) -> Self {
        Reader {
            input,
            pos: 0,
            base: 0,
            depth: 0,
        }
    }

    /// Absolute offset of the cursor within the original document.
    ///
    /// # Returns
    ///
    /// The offset the next tag octet would be read from.
    pub fn offset(&self) -> usize {
        self.base.saturating_add(self.pos)
    }

    /// Report whether every byte in this reader's slice has been consumed.
    ///
    /// # Returns
    ///
    /// `true` when no bytes remain.
    pub fn is_empty(&self) -> bool {
        self.pos >= self.input.len()
    }

    /// Number of unconsumed bytes.
    ///
    /// # Returns
    ///
    /// `input.len() - pos`, saturating at zero.
    pub fn remaining(&self) -> usize {
        self.input.len().saturating_sub(self.pos)
    }

    /// Nesting depth of this reader; 0 for the top-level document.
    ///
    /// # Returns
    ///
    /// The depth assigned when the reader was created.
    pub fn depth(&self) -> usize {
        self.depth
    }
}
