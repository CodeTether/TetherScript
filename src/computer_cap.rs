//! Computer-use capability backed by CodeTether desktop action envelopes.
//!
//! This is the language-side seam for harness-granted desktop automation. It
//! serializes tetherscript method calls to a local bridge instead of binding the
//! interpreter to Win32, macOS Accessibility, or X11 APIs directly.

#[path = "computer_cap/actions.rs"]
mod actions;
#[path = "computer_cap/authority.rs"]
mod authority;
#[path = "computer_cap/authority_trait.rs"]
mod authority_trait;
#[path = "computer_cap/call.rs"]
mod call;
#[path = "computer_cap/describe.rs"]
mod describe;
#[path = "computer_cap/invoke.rs"]
mod invoke;
#[path = "computer_cap/mapping.rs"]
mod mapping;
#[path = "computer_cap/response.rs"]
mod response;
#[path = "computer_cap/scope.rs"]
mod scope;
#[path = "computer_cap/scope_apply.rs"]
mod scope_apply;
#[path = "computer_cap/scope_narrow.rs"]
mod scope_narrow;
#[path = "computer_cap/transport.rs"]
mod transport;
#[path = "computer_cap/value.rs"]
mod value;

pub use authority::ComputerAuthority;
