# Contributing

Thanks for working on tetherscript. Keep changes focused, reviewable, and
aligned with the runtime semantics already in the repository.

## Development Rules

- Use lowercase `tetherscript` in user-facing language copy.
- Keep the language dynamically typed and runtime ownership checked.
- Keep the default core crate free of third-party dependencies. External crates
  belong behind explicit features or in separate adapters/examples, and every
  exception requires a concrete design reason.
- Do not replace the native browser agent path with Chromium, Chrome DevTools,
  or Playwright.
- Add tests for every behavior change.
- Preserve the examples unless the change intentionally updates their behavior.

## Build Prerequisites

The default build requires only Rust. The optional `openssl-tls` feature vendors
OpenSSL and therefore requires Perl plus the platform C toolchain when built
from source; no OpenSSL installation is needed at runtime. On Windows, use a
native Windows Perl distribution rather than MSYS Perl.

## Required Checks

Run these before sending a release or pull request:

```bash
bash ./check_file_limits.sh
cargo fmt --check
cargo clippy -- -D warnings
cargo test
cargo test --features tera
cargo test --doc
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps
cargo package
# Requires Perl and a platform C toolchain:
cargo test --features openssl-tls
```

On Windows without Bash, run the equivalent PowerShell checker:

```powershell
.\check_file_limits.ps1
```

For local feature work, also run the relevant `.tether` examples through the
`tetherscript` binary.

## Release Checklist

- Bump the crate version in `Cargo.toml` and `Cargo.lock`.
- Confirm `cargo publish --dry-run` passes.
- Confirm the package contents are intentional with `cargo package --list`.
- Publish only from a clean, committed worktree.
