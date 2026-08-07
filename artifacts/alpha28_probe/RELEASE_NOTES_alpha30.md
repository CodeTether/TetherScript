## Added

### Bitwise operators

alpha.29 had none: `|`, `^`, and `~` were lexer errors, `<<` and `>>` were parse
errors, and `&` between two expressions was rejected outright. Flag masking,
protocol framing, and hashing all had to be faked with arithmetic.

`& | ^ ~ << >>` now work on both backends, with Rust's precedence: `|` loosest,
then `^`, then `&`, then the shifts, all tighter than a comparison.

```tether
println(12 & 10)     // 8
println(12 | 10)     // 14
println(12 ^ 10)     // 6
println(1 << 4)      // 16
println(-8 >> 1)     // -4  (arithmetic shift, sign preserved)
println(~0)          // -1
println(1 | 2 ^ 3 & 4)   // 3, same as Rust and Python
```

Infix `&` is bitwise AND; prefix `&` is still a borrow. Position disambiguates
them, exactly as in Rust, so both readings coexist in one expression:

```tether
let borrowed = &xs
let masked   = flags & 3
```

Two deliberate departures from Rust:

- **Operands must be integers.** Rust accepts `bool & bool` as a
  non-short-circuiting logical and. Here it is an error, because in a language
  that has `&&`, that spelling is nearly always a mistyped `&&`, and silently
  accepting it is how a short-circuit bug ships.
- **Shift counts are validated.** Rust panics in debug and masks in release.
  `1 << 64` reports the count and the limit instead.

Semantics live in the shared `apply_binary`/`apply_unary`, so the VM and the
tree-walking interpreter cannot drift. Every case is asserted on both.

See [`examples/bitwise.tether`](examples/bitwise.tether).

### UDP sockets

There was no datagram support of any kind. `resource.udp_bind(host, port)` is now
an owned move-only resource with `send_to`, `recv_from`, `local_addr`, and `port`:

```tether
let socket = resource.udp_bind("127.0.0.1", 0)?
socket.send_to("ping", "127.0.0.1", peer_port)?
let datagram = socket.recv_from(1024)?   // { bytes, from }
```

A received datagram is a `{ bytes, from }` map. UDP is connectionless, so the
sender belongs to the payload rather than the socket; returning them together
spares the caller correlating two calls.

See [`examples/udp_echo.tether`](examples/udp_echo.tether).

### `--grant-tcp` and `--grant-udp`

```bash
tetherscript run --grant-tcp 127.0.0.1:8080 server.tether
tetherscript run --grant-udp '*' dns_client.tether
```

Repeatable `host`, `host:port`, or `*` patterns. TCP and UDP are granted
separately, and `--access-mode full` grants both.

## Fixed

**Sockets were ambient authority.** `resource.tcp_listen` and
`resource.tcp_connect` bound and dialed with no grant whatsoever, so a script
running under the default `--access-mode restricted` had unrestricted network
reach. This was the gap reported against alpha.29.

Sockets are now **denied by default**:

```
$ tetherscript run server.tether
tetherscript: resource.tcp_listen: TCP access requires
`tetherscript run --grant-tcp <host[:port]>` or `--access-mode full`
```

- Denials name the flag that would allow the call.
- Out-of-scope denials are worded differently from no-grant-at-all, so "you
  granted nothing" is distinguishable from "you granted something narrower".
- A UDP grant does not authorize TCP, and vice versa.
- UDP sends re-check the destination, so a socket bound under a narrow grant
  cannot be reused to reach an address outside it.
- Malformed patterns fail at startup, not at first use.

The grant is thread-local and consulted at the syscall boundary rather than
threaded through every resource factory. That is a deliberate tradeoff: owned
resources are constructed deep in `value::resource` with no capability handle in
scope, and restructuring every factory signature inside a security fix would have
been a much larger, riskier diff. The check is deny-by-default, so the failure
mode of the shortcut is refusal, not leakage.

## Install

```
cargo add tetherscript@0.1.0-alpha.30
```

## Validation

All ten gates in `.github/workflows/ci.yml`, **observed green on GitHub Actions**:

https://github.com/CodeTether/TetherScript/actions/runs/31133278908

`check_file_limits.sh` · `fmt --check` · `clippy -D warnings` · `browser_wpt_like`
· `browser_wpt_json` · `browser_wpt_upstream` · `cargo test` · `cargo test --doc`
· `cargo doc -D warnings` · `cargo package`

Locally on the tagged commit: **4218 tests passed, 0 failed**; 674 doc tests
passed. All 106 `.tether` files under `examples/` and `tests/` still parse. New
coverage: 9 bitwise integration tests asserting both backends agree, 8 UDP/grant
integration tests driving the real binary, 15 `socket_cap` unit tests, and a case
asserting the socket denial actually fires so the gate cannot silently regress to
a no-op.

Registry evidence: crates.io reports `max_version` and `default_version` as
`0.1.0-alpha.30`, not yanked.

## Breaking change

Scripts that opened TCP sockets without a grant now fail. This is the point of the
release. Add `--grant-tcp <host[:port]>` — or `--access-mode full` for the old
behavior.

## Still open

- `fs_*` and `process_*` remain ambient: `fs_read` still succeeds without
  `--grant-fs`. Prefer the `fs` capability object, which enforces correctly and is
  undefined unless granted. Sockets are no longer in this category.
- No TLS for socket resources.
- `Rule::resource_types` in the ad-block layer is parsed but not consulted during
  matching, so `$type` modifiers widen to "any resource type".
