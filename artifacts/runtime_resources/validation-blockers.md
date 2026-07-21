# Runtime resource validation blockers

Recorded on 2026-07-21 after the owned-resource implementation.

`cargo test` passed the 865-test library target and every integration target
before `tests/agent_tui.rs`, where these existing TUI/provider tests failed:

- `agent_tui_exits_on_stdin_eof_instead_of_redrawing_forever`
  - timed out waiting for the TUI to exit on stdin EOF
  - reproduced when run alone
- `agent_tui_plain_answer_uses_one_provider_request`
  - expected one provider request, observed zero
- `agent_tui_sends_tools_and_executes_model_tool_call`
  - expected stdout to contain `TLS cwd`

A second full run skipped only those three cases. All tests through the TUI
target passed, then `agent_tui_drives_persistent_native_browser_tools` failed
because the default build does not enable the optional `openssl-tls` feature:

```text
tetherscript: vault: TLS connector failed: TLS support requires the `openssl-tls` feature
```

The focused resource test commands, clippy, doc tests, and rustdoc all passed.
No TUI, provider, vault, or TLS implementation was changed to mask these
unrelated failures.
