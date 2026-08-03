//! Deterministic A/B test assignment.
//!
//! Owner: sub-agent `ab_test`. The reference application registers an A/B test
//! middleware in its Actix `create_app`; the tetherscript port had no equivalent.
//! This group supplies the decision half of that middleware as pure built-ins, so
//! a handler can ask "which variant does this visitor see?" without owning any
//! mutable global state.
//!
//! # Assignment is a hash, never a random number
//!
//! `abtest_bucket::bucket` hashes `(seed, subject)` with the in-tree SHA-256 and
//! derives a bucket in `0..10000`. There is deliberately **no** random number
//! generator anywhere in this group. A visitor who sees variant `A` on one request
//! and `B` on the next invalidates the experiment and looks broken to the user, so
//! assignment must be reproducible across requests, across processes, and across
//! restarts. Given the same seed and subject the answer is the same forever.
//!
//! # Weight convention
//!
//! Weights are **integer percentage points that must sum to exactly 100**, and a
//! configuration that violates that is rejected by `ab_experiment` at construction
//! time. Silently normalising a bad sum would skew traffic for the whole life of
//! the experiment and the mistake would only show up as a confusing result. Zero
//! variants, a negative weight, and a duplicate variant name are rejected for the
//! same reason.
//!
//! # Security
//!
//! A subject identifier is attacker-controlled: it arrives from a cookie, a header,
//! or a query string. It is used here only as hash input.
//!
//! * **Never** interpolate a subject id into a filesystem path or an SQL fragment.
//!   Nothing in this group escapes or validates it beyond requiring a non-empty str.
//! * **Never** treat a variant assignment as an authorisation decision. A caller
//!   who chooses its own subject id can grind ids until it lands in the variant it
//!   wants, so "is in variant B" must never gate access to anything.
//!
//! # Script surface
//!
//! Names carry an `ab_` prefix, matching how `bucket_new` and `cookie_parse` are
//! spelled, so nothing collides with ordinary script bindings.
//!
//! | Builtin | Returns |
//! |---|---|
//! | `ab_experiment(config)` | `Result` of a validated experiment map |
//! | `ab_assign(experiment, subject)` | `Result` of the variant name str |
//! | `ab_assign_from_request(experiment, request)` | `Result` of an assignment map |
//! | `ab_bucket(seed, subject)` | int in `0..10000` |
//!
//! # Examples
//!
//! ```tether
//! let cfg = map()
//! cfg.name = "checkout_button"
//! cfg.seed = "checkout_button_v1"
//! cfg.sticky_cookie = "ab_checkout"
//! let a = map()
//! a.name = "control"
//! a.weight = 50
//! let b = map()
//! b.name = "green"
//! b.weight = 50
//! cfg.variants = [a, b]
//!
//! let exp = ab_experiment(cfg)?
//! println(ab_assign(exp, "visitor-91af")?)
//! ```
//!
//! # Layout
//!
//! * `abtest_args` — argument and field coercion with named errors
//! * `abtest_bucket` — the SHA-256 bucket function
//! * `abtest_variant` — variant list parsing
//! * `abtest_weight` — the weight invariants: type, sign, uniqueness, sum
//! * `abtest_config` — the experiment shape, revalidated on every use
//! * `abtest_assign` — mapping a bucket onto a variant
//! * `abtest_cookie` — sticky-cookie lookup from a request map
//! * `abtest_cookie_header` — raw `Cookie` header lookup and splitting
//! * `abtest_request` — the request-driven assignment, existing cookie wins
//! * `abtest_shape` — the assignment map a script sees
//! * `abtest_echo` — the canonical experiment map `ab_experiment` returns
//! * `abtest_ops` — thin adapters from `Value` into the logic above
//! * `abtest_install` — built-in registration

use std::cell::RefCell;
use std::rc::Rc;

use crate::value::Env;

#[path = "abtest_args.rs"]
mod abtest_args;
#[path = "abtest_assign.rs"]
mod abtest_assign;
#[path = "abtest_bucket.rs"]
mod abtest_bucket;
#[path = "abtest_config.rs"]
mod abtest_config;
#[path = "abtest_cookie.rs"]
mod abtest_cookie;
#[path = "abtest_cookie_header.rs"]
mod abtest_cookie_header;
#[path = "abtest_echo.rs"]
mod abtest_echo;
#[path = "abtest_install.rs"]
mod abtest_install;
#[path = "abtest_ops.rs"]
mod abtest_ops;
#[path = "abtest_request.rs"]
mod abtest_request;
#[path = "abtest_shape.rs"]
mod abtest_shape;
#[path = "abtest_variant.rs"]
mod abtest_variant;
#[path = "abtest_weight.rs"]
mod abtest_weight;

/// Register this group's built-ins.
///
/// # Arguments
///
/// * `env` — Global environment the interpreter is populating.
///
/// # Returns
///
/// Nothing. `ab_experiment`, `ab_assign`, `ab_assign_from_request`, and
/// `ab_bucket` are defined in `env` as immutable bindings.
pub(crate) fn install(env: &Rc<RefCell<Env>>) {
    abtest_install::install(env);
}
