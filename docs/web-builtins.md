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
| template | `html_escape`, `html_attr`, `template_render`, `template_render_raw`, `template_render_inherited` |
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

A dispatcher builds a table of patterns and tries each against `req.path`, so one handler
serves every id.

## Templating

`template_render` covers the Tera subset a real view leans on:

| Construct | Notes |
| --- | --- |
| `{{ value }}` / `{{{ raw }}}` | Escaped by default; triple braces emit raw |
| `{% if %}` / `{% elif %}` / `{% else %}` | Conditions may compare: `==`, `!=`, `>`, `<`, `>=`, `<=` |
| `{% for x in items %}` | Iterates a list; the binding does not leak past `endfor` |
| `{% block %}` / `{% extends %}` | Inheritance, via `template_render_inherited` |
| `{% include %}` | Splices a partial, with `ignore missing` |
| `{# comment #}` | Dropped entirely |

```tether
let page = template_render(
    "\{% if user %\}<h1>\{\{ user.name \}\}</h1>\{% endif %\}" +
    "<ul>\{% for i in items %\}<li>\{\{ i \}\}</li>\{% endfor %\}</ul>",
    context,
)?
```

Condition truthiness follows Tera/Jinja rather than tetherscript: an empty list,
empty string, and zero are all false, so `{% if items %}` means "is there anything
to show". A **missing** key is also false, since that is how a view tests an
optional value — but a missing key in a `{{ }}` hole is still an error, because
there a typo would silently blank the page.

### Conditions

A condition is a bare key tested for truthiness, or a comparison:

```tether
\{% if step.id == current_step.id %\}...\{% endif %\}
\{% if count > 3 %\}...\{% endif %\}
```

Both sides may be dotted paths or literals. A missing key compares as `nil` rather
than erroring, matching the tolerance a bare key gets. Ordering (`<`, `>`) requires
numbers: comparing strings lexicographically is almost always a mistake in a
template, so it is refused by name instead.

### Inheritance

`template_render_inherited(template, context, templates)` resolves
`{% extends %}`. The child names a parent and supplies `{% block %}` bodies; the
parent renders, with the child's blocks substituted by name. A block the child does
not override keeps the parent's content, so a block body is a default rather than a
requirement.

```tether
let templates = map()
templates["layout"] = "<html><title>\{% block title %\}Site\{% endblock %\}</title>" +
                      "<body>\{% block content %\}\{% endblock %\}</body></html>"

let page = "\{% extends \"layout\" %\}\{% block content %\}<h1>\{\{ heading \}\}</h1>\{% endblock %\}"
let html = template_render_inherited(page, context, templates)?
```

Chains nest to any depth up to 16, and the most-derived template wins at every
level. Exceeding the depth is reported as a possible cycle rather than looping.
Both quote styles work, and `{% endblock name %}` is accepted alongside bare
`{% endblock %}`.

Templates come from a caller-supplied **map**, not the filesystem: `template_*` are
pure built-ins, so reading files from inside them would bypass the `fs` capability.
A host that wants on-disk views reads them through `fs` and passes the map in.

### Filters

`{{ value | filter | filter(arg=x) }}` chains left to right.

| Filter | Effect |
| --- | --- |
| `safe` | Emit raw, suppressing escaping |
| `default(value=..)` | Substitute when the key is missing **or** `nil` |
| `json`, `json_encode`, `to_json` | Encode as JSON |
| `length` | Length of a str, list, or map |
| `upper`, `lower`, `trim` | String transforms |
| `escape`, `html_attribute_encode` | Explicit escaping |
| `int`, `float`, `str` | Coercion, refusing what cannot convert |
| `first`, `last` | List ends; `nil` when empty |
| `round` | Nearest integer |
| `truncate(length=N, end="..")` | Shorten, counting characters not bytes |
| `date(format="%b %d, %Y")` | strftime over Unix seconds |

A separator inside a quoted argument is data, so `date(format="%b %d, %Y")` keeps its
comma and a `|` inside a literal does not split the pipeline.
`{{ data | json | safe }}` is the idiom for embedding a value in a `<script>` block,
and is why `safe` exists: it marks content as intentionally raw. Everything else
escapes, so reaching for `safe` is a visible decision rather than a default.

`default` fires for a missing key *and* for `nil`, matching Tera. Its argument is
validated on every render even when the value is present, so a malformed
`default()` surfaces immediately rather than only on the rows where the key happens
to be absent.

An unknown filter is an error, never a pass-through: silently ignoring `| json`
would emit a bare value where a page expects JSON and break the consuming script
rather than the render.

### Still unsupported

`{% macro %}` and `{% set %}`, plus application-specific filters. Each is reported by
name — with a hint where there is an obvious alternative — rather than ignored, so a
template using one fails loudly instead of rendering a hole.

Application filters (the reference has `clean_llm_meta`) are deliberately not
registerable. They belong to the application, so a script computes them into the
context before rendering. That keeps the engine's behaviour identical everywhere
rather than varying with what a caller happened to register.

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
- No template macros or `{% set %}`, and no registerable application filters.
  `template_render` covers substitution, `if`/`elif`, comparisons, `for`, comments, and
  the common filters; `template_render_inherited` adds `extends`, `block`, and
  `include`. The optional `tera` feature remains the richer path.
- No regex engine, so `validate` uses hand-written scanners and `is_email` is a
  pragmatic filter, not RFC 5322 and not proof of deliverability.
- No WebSocket upgrade.