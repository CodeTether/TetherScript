# Runtime resource validation blockers

Recorded on 2026-07-21 after the owned-resource implementation.

`cargo test` passed the 869-test library target and every integration target
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

A later all-target continuation also reached
`embedded_vault::vault_is_only_visible_to_authority_scripts`, which fails for
the same disabled optional TLS feature and the same diagnostic.

The same default-feature limitation also blocks these stdio provider tests:

- `stdio_agent_keeps_jsonrpc_on_stdout`
- `stdio_agent_tools_can_edit_workspace`

`tera_example_matches_golden_output` is independently blocked because the
default build disables the optional `tera` feature while the integration test
still expects `tera_render` to be available.

The safe release-example sweep also found that `examples/json.tether` still
uses single-quoted strings, which the lexer rejects at line 4. Core, ownership,
owned-resource, interpolation, and async examples passed; the use-after-move
example produced its expected nonzero ownership error.

The focused resource test commands, clippy, doc tests, and rustdoc all passed.
No TUI, provider, vault, or TLS implementation was changed to mask these
unrelated failures.

Revalidated after recursive move-only transfer enforcement was added: the same
three `agent_tui` failures reproduced, while all 869 library tests and all
focused owned-resource tests passed.
