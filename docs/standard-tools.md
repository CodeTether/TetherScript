# TetherScript Standard Tools

TetherScript's trusted CLI/runtime should include these scripting primitives. Host tools are intentionally not installed in the sandboxed `eval()` runtime. The language crate is zero-dependency: all standard tools are implemented with Rust `std` and in-tree code.

## Core Values

- `print(...)`, `println(...)`: write text output.
- `len(value)`: length for strings, lists, and maps.
- `type_of(value)`: return TetherScript's runtime type name.
- `str(value)`: convert any value to its display string.
- `parse_int(value)`, `parse_float(value)`: parse numeric text and return `Result`.
- `assert(condition[, message])`: stop execution when an invariant is false.
- `sha256_hex(text)`: compute a SHA-256 hex digest with TetherScript's std-only implementation.
- `base64_encode(text)`, `base64_decode(text)`: encode/decode Base64 text with TetherScript's std-only implementation.
- `Ok(value)`, `Err(message)`: construct `Result` values for `?` propagation.
- `map()`: construct an empty map.

## Collections And Text

- String methods: `.len()`, `.upper()`, `.lower()`, `.trim()`, `.contains(s)`, `.starts_with(s)`, `.ends_with(s)`, `.replace(from, to)`, `.split(sep)`, `.lines()`.
- List methods: `.len()`, `.push(value)`, `.pop()`, `.join(sep)`, `.contains(value)`.
- Map methods: `.len()`, `.keys()`, `.values()`, `.contains(key)`.

## Data Formats

- `json_parse(text)`: parse JSON into TetherScript values.
- `json_encode(value)`: encode compact JSON.
- `json_encode_pretty(value)`: encode formatted JSON.

## Host Runtime

- Time: `time_now_ms()`, `sleep_ms(ms)`.
- Environment: `env_get(name)`.
- Process control: `process_args()`, `process_pid()`, `process_platform()`, `process_arch()`, `process_list()`, `process_kill(pid[, force])`.
- OS metadata: `os_platform()`, `os_arch()`, `os_tmpdir()`, `os_homedir()`, `os_eol()`.
- Working directory: `cwd()`, `chdir(path)`.
- Filesystem: `fs_read(path)`, `fs_write(path, body)`, `fs_exists(path)`, `fs_list(path)`, `fs_mkdir(path)`, `fs_stat(path)`, `fs_remove(path)`, `fs_rename(from, to)`, `fs_copy(from, to)`.
- Path: `path_sep()`, `path_join(parts)`, `path_dirname(path)`, `path_basename(path)`, `path_extname(path)`, `path_normalize(path)`, `path_resolve(path)`.
- URL: `url_parse(url)`.
- Process execution: `process_run(command[, args[, stdin[, timeout_ms]]])`.
- Terminal UI: `tui_size()`, `tui_render(view)`, `tui_present(view)`, `tui_read_event([prompt])`, `tui_clear()`, `tui_cursor(visible)`, `tui_alt_screen(enabled)`, `tui_move_to(row, col)`.
- Stdio protocol: `stdio_read()`, `stdio_write(value)`, `stdio_write_err(text)`, `jsonrpc_request(id, method, params)`, `jsonrpc_response(id, result)`, `jsonrpc_error(id, code, message[, data])`, `jsonrpc_notify(method, params)`.
- HTTP: `http_get(url)`, `http_head(url)`, `http_post(url, body)`, `http_request(method, url[, body[, headers]])`, `http_serve(port, handler)`, `http_serve_static(port, root_dir)`.
- Browser runtime: `browser_parse_html(html)`, `browser_parse_css(css)`, `browser_styles(html[, css])`, `browser_query_selector(html, selector)`, `browser_text_content(html, selector)`, `browser_snapshot(html[, css[, width]])`, `browser_layout(html[, css[, width]])`, `browser_display_list(html[, css[, width]])`, `browser_render(html[, css[, width]])`, `js_eval(source)`, `browser_eval_js(html, script)`, `browser_run_scripts(html)`, `browser_compatibility_report()`. The browser JS host exposes DOM querying/mutation, synchronous event listeners/property handlers, `this`, `typeof`, function expressions, `location`, and `navigator`.
- SMTP: `smtp_send(host, port, from, to, subject, body)`.

## Provider Vault Bootstrap

`tetherscript run --grant-provider-vault <provider-id> script.tether` grants the
normal `provider` capability from HashiCorp Vault KV v2 secrets using the same
bootstrap environment names as CodeTether-agent:

- `VAULT_ADDR`
- `VAULT_TOKEN`
- `VAULT_MOUNT` (default `secret`)
- `VAULT_SECRETS_PATH` (default `codetether/providers`)

The secret is read from `<mount>/data/<path>/<provider-id>`. The loader accepts
`api_key`, `base_url`, `organization`, and `headers`. `api_key` becomes a hidden
`Authorization: Bearer ...` bound header and is not visible to tetherscript code.
CodeTether `openai-codex` OAuth secrets from `codetether auth codex` are also
accepted; tetherscript binds the access token and ChatGPT account id and uses
the ChatGPT Codex Responses endpoint for streamed text responses.

`--access-mode full` is a convenience for local agent runs. A script can also
declare the same local agent authority with a top-of-file header:
`// tetherscript: authority agent`. This grants filesystem authority to the
current directory and loads provider secrets from Vault first.
Agent scripts can opt into turn-boundary reloads with
`// tetherscript: hot-reload`. The script writes `.tetherscript/reload` when it
intentionally exits for reload, and the runner restarts the same source only when
that marker matches the current script path.
When `CODETETHER_DEFAULT_MODEL` is set to `provider/model`, full access prefers
that Vault provider before falling back to the supported provider order. For
local development, if no Vault default is configured, it falls back to
Windows/process environment variables such as `OPENAI_API_KEY`,
`OPENROUTER_API_KEY`, `CEREBRAS_API_KEY`, or `ZAI_API_KEY`. Set
`CODETETHER_DISABLE_ENV_FALLBACK=1` to require Vault-configured providers only.
Set `TETHERSCRIPT_PROVIDER` or `TETHERSCRIPT_AGENT_PROVIDER` to choose a
specific provider id.

## Safety Defaults

- Fallible host tools return TetherScript `Result` values so scripts can use `?`, `.unwrap()`, `.is_ok()`, and `.err()`.
- `process_run` executes a command plus argv directly; it does not invoke a shell unless the script explicitly runs one.
- `process_list` returns PID/name maps; `process_kill` may fail when the OS denies access or the PID no longer exists.
- File and process output are bounded to keep accidental large reads from consuming unbounded memory.
- Std-only HTTP supports plain `http://`; `https://` requires TLS and is rejected explicitly.
- The standard-tool implementation itself uses Rust `std` only. It does not add or depend on external crates for JSON, LSP JSON-RPC framing, SMTP DKIM signing, path, filesystem, process, Base64, SHA-256, or URL parsing.
