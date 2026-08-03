//! Password hashing built-ins: `password_hash`, `password_verify`, and
//! `password_needs_rehash`.
//!
//! A real application needs credential storage, and the tree had no password
//! hashing primitive at all — only raw digests, which must never be used for
//! passwords because they are fast by design.
//!
//! # Script surface
//!
//! | Builtin | Returns |
//! |---|---|
//! | `password_hash(password)` | `Result` of the PHC-style str |
//! | `password_verify(password, encoded)` | `Result` of bool |
//! | `password_needs_rehash(encoded, min_iterations)` | `Result` of bool |
//!
//! # Stored format
//!
//! ```text
//! $pbkdf2-sha256$i=600000$<base64 salt>$<base64 hash>
//! ```
//!
//! The string is self-describing: algorithm, cost, and salt travel with the
//! digest. That is what allows the cost to be raised later without invalidating
//! stored credentials — an old hash still verifies at its own recorded count, and
//! `password_needs_rehash` flags it for upgrade on the next successful login.
//!
//! # Why PBKDF2 and not Argon2id
//!
//! **Argon2id is the stronger choice** and is what a greenfield system should use.
//! It is memory-hard, so a GPU or ASIC attacker cannot trade cheap parallelism for
//! speed the way it can against PBKDF2. PBKDF2 is only CPU-hard.
//!
//! PBKDF2-HMAC-SHA-256 is used here for one reason: this crate's core build has
//! **zero required dependencies** (see AGENTS.md), and adding `argon2` would break
//! that. PBKDF2 is buildable from the in-tree SHA-256, is specified in RFC 8018,
//! and remains an accepted choice — NIST SP 800-63B and OWASP both still list it,
//! the latter at 600,000 iterations for SHA-256, which is the default here.
//!
//! So this is a deliberate dependency-versus-strength tradeoff, not an assertion
//! that PBKDF2 is the best available KDF. If an Argon2id dependency ever becomes
//! acceptable, the PHC-style encoding above already carries an algorithm field, so
//! a second scheme can be added and old hashes upgraded on login.
//!
//! # Security properties
//!
//! * A fresh 16-byte salt per call, from `/dev/urandom` where available, so two
//!   users with the same password never share a hash.
//! * Verification compares digests in constant time via the `hmac` group's
//!   `constant_time_eq`, so a mismatch cannot be located byte by byte.
//! * A malformed stored hash is an `Err`, not a silent `false`, so database
//!   corruption is distinguishable from a wrong password.
//!
//! # Examples
//!
//! ```tether
//! let stored = password_hash("correct horse battery staple")?
//! if password_verify("correct horse battery staple", stored)? {
//!     println("authenticated")
//! }
//! if password_needs_rehash(stored, 1200000)? {
//!     println("cost raised; rehash on next login")
//! }
//! ```
//!
//! # Layout
//!
//! * `password_args` — argument coercion
//! * `password_install` — environment registration
//! * `password_ops` — hash, verify, and rehash decisions
//! * `password_phc` — PHC-style encode and parse
//! * `password_pbkdf2` — RFC 8018 PBKDF2-HMAC-SHA-256, any output length
//! * `password_salt` — per-password salt generation

use std::cell::RefCell;
use std::rc::Rc;

use crate::value::Env;

// Explicit paths: the parent module is declared with `#[path]`, so relative
// submodule resolution would otherwise look in `src/` directly.
#[path = "password_args.rs"]
mod password_args;
#[path = "password_install.rs"]
mod password_install;
#[path = "password_ops.rs"]
mod password_ops;
#[path = "password_pbkdf2.rs"]
mod password_pbkdf2;
#[path = "password_phc.rs"]
mod password_phc;
#[path = "password_salt.rs"]
mod password_salt;

/// Register this group's built-ins.
///
/// # Arguments
///
/// * `env` — Global environment the interpreter is populating.
pub(crate) fn install(env: &Rc<RefCell<Env>>) {
    password_install::install(env);
}
