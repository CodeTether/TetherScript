# Web built-ins

Primitives for building HTTP services in tetherscript, added so a real
application port does not have to hand-roll them per handler. Every group is
in-tree and needs no dependency and no feature flag.

Each group registers through `src/web_builtins.rs`, one module per concern, so a
group can be added without touching a shared registration list.

## Groups

| Group | Built-ins |
| --- | --- |
| hmac | `hmac_sha256_hex`, `hex_encode`, `hex_decode`, `constant_time_eq` |
| jwt | `jwt_sign`, `jwt_verify`, `jwt_decode_unverified` |
| cookie | `cookie_parse`, `cookie_serialize` |
| session | `session_sign`, `session_verify`, `session_touch`, `session_expired` |
| csrf | `csrf_token`, `csrf_verify`, `csrf_claims` |
| password | `password_hash`, `password_verify`, `password_needs_rehash` |
| random | `random_bytes_hex`, `random_token`, `random_int`, `random_choice` |
| uuid | `uuid_v4`, `uuid_parse`, `uuid_is_valid` |
| base32 | `base32_encode`, `base32_encode_nopad`, `base32_decode` |
| form | `url_encode`, `url_decode`, `form_parse`, `form_encode` |
| multipart | `multipart_parse`, `multipart_field`, `multipart_boundary` |
| route | `route_match`, `route_params`, `path_segments` |
| header | `header_get`, `bearer_token`, `client_ip`, `accepts`, `security_headers` |
| mime | `mime_for_path`, `mime_parse`, `mime_is_text` |
| etag | `etag_of`, `etag_weak`, `etag_matches`, `cache_control`, `not_modified_response` |
| datetime | `http_date`, `http_date_parse`, `rfc3339`, `rfc3339_parse`, `time_now_secs` |
| template | `html_escape`, `html_attr`, `template_render`, `template_render_raw` |
| validate | `is_email`, `is_slug`, `is_digits`, `normalize_phone`, `validate_fields` |
| ratelimit | `bucket_new`, `bucket_take`, `retry_after_header`, `too_many_requests_response` |
| sse | `sse_event`, `sse_comment`, `sse_retry` |
| log | `log_json`, `log_info`, `log_warn`, `log_error`, `log_debug`, `log_level_enabled` |

## Braces in patterns and templates

`{` opens a string interpolation hole, so any literal brace must be escaped. This
bites two groups in particular:

```tether
// Route patterns: write \{code\}, not {code}
let params = route_match("/api/short-urls/\{code\}", req.path)?

// Templates: write \{\{ name \}\}
let page = template_render("<h1>\{\{ title \}\}</h1>", context)?
```

Without the escapes the lexer reads `{code}` as an interpolation of a variable
named `code` and fails with `undefined variable`, or `{}` as an empty
interpolation. This is the single most common mistake when using these groups.

## Routing

`route_match` returns a captures map when the pattern applies, `nil` when it does
not, and an error only when the *pattern* is malformed. A non-match is an ordinary
outcome, since a dispatcher tries many patterns per request.

```tether
// nil, not an error, when the path does not match
let captured = route_match("/customers/\{id\}", "/healthz")?
```

`{name}` cannot match across a `/`, and a trailing `{name:.*}` captures the
remainder including slashes. Path segments are percent-decoded *after*
segmentation, so an encoded `%2F` can never forge a separator.

See `examples/the reference application/server/router.tether` for a dispatch table built on
this.

## Security defaults

These are deliberate, and the tests assert them:

- **`template_render` escapes by default.** `{{ name }}` is HTML-escaped; only the
  explicit `{{{ name }}}` form emits raw markup. A renderer that interpolates
  untrusted values unescaped is an XSS vector.
- **`jwt_verify` picks the algorithm.** `alg: none` and every non-HS256 value are
  rejected. Dispatching on the token's own `alg` is the classic JWT forgery.
- **Signature comparisons are constant-time** in jwt, session, csrf, and password.
- **`cookie_serialize` rejects** control characters, semicolons, and newlines in
  the name, the value, and every attribute, so no option can inject a second
  cookie.
- **`log_json` writes to stderr**, because stdout already carries HTTP response
  bodies, captured `println` output, and JSON-RPC frames.
- **`bearer_token` refuses a bare token** with no scheme.
- **`client_ip` honors `X-Forwarded-For`**, which is client-controlled. Trust it
  only behind a proxy that overwrites it; otherwise a caller can forge its own
  address and defeat per-IP rate limiting.

## Session cookies

`session_sign` produces a signed, *not encrypted*, value: anyone holding the
cookie can read the payload, so no secret belongs in it. This matches the
reference behaviour it was ported from, where a Node service reads the same
session id.

```tether
let payload = map()
payload.uid = "u-7"
payload.exp = time_now_secs() + 604800

let value = session_sign(payload, secret)?
let opts = map()
opts.http_only = true
opts.same_site = "Lax"
opts.path = "/"
opts.expires = http_date(payload.exp)?
let header = cookie_serialize("sid", value, opts)?
```

The signature covers the encoded payload exactly as transmitted. Verification
never re-serializes, because `map` is unordered and re-encoding could produce
different bytes than the signer wrote — which would reject valid cookies
intermittently.

## Rate limiting

Bucket state belongs to the caller, so there is no hidden global. `bucket_take`
returns the decision *and* the next bucket; a caller that does not persist the
returned bucket has no limit at all.

```tether
let taken = bucket_take(bucket, 1)?
if !taken.allowed {
    return too_many_requests_response(taken.retry_after_ms)
}
let bucket = taken.bucket  // must be kept
```

## Password hashing

`password_hash` uses PBKDF2-HMAC-SHA256 with a random per-password salt and
records the iteration count in a PHC-style string, so the cost can be raised later
without invalidating stored hashes. Argon2id would be stronger; PBKDF2 was chosen
because the core build takes no dependencies.

## What is not here

- No Redis or any session *store*; only the signed-cookie half.
- No Tera-compatible template inheritance, filters, or loops. `template_render`
  does substitution and escaping. The optional `tera` feature remains the richer
  path.
- No regex engine, so `validate` uses hand-written scanners and `is_email` is a
  pragmatic filter, not RFC 5322 and not proof of deliverability.
- No WebSocket upgrade.
