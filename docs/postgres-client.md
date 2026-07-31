# Native PostgreSQL client

`tetherscript::postgres` speaks the PostgreSQL v3 frontend/backend protocol
directly over TCP. It exists so the zero-dependency core can reach a real
database: before it, the `db` capability described in
[standard-tools.md](standard-tools.md) required the host to supply a driver such
as SQLx.

Drivers are still supported and still welcome — `QueryHandler` is unchanged. This
client is simply an in-tree implementation a host may use instead.

## Usage

```rust,no_run
use tetherscript::postgres::{Config, Connection};

let mut connection = Connection::connect(&Config {
    host: "127.0.0.1".into(),
    port: 5432,
    user: "tsuser".into(),
    password: "pencil".into(),
    database: "tsdb".into(),
})?;

let rows = connection.simple_query("SELECT id, name FROM users ORDER BY id")?;
```

Nothing is read from the ambient environment. The host decides where credentials
come from, so the client never reaches for `DATABASE_URL` on its own.

## Row decoding

Rows are a list of maps keyed by column name. The simple-query protocol returns
every field as text with no type information, so values are converted by shape:

| Text field | tetherscript value |
| --- | --- |
| `t` / `f` | bool |
| integer literal | int |
| decimal literal | float |
| field length -1 | `nil` (SQL NULL) |
| anything else | str |

This is a heuristic, not type-driven decoding. A column whose text happens to
look numeric decodes as a number. When the exact SQL type matters, cast in the
query — for example `SELECT id::text`.

## Authentication

Negotiates whichever method the server requests:

| Method | Status |
| --- | --- |
| trust (`AuthenticationOk`) | supported |
| cleartext `password` | supported |
| `md5` | supported (legacy) |
| SCRAM-SHA-256 | supported, preferred |
| GSSAPI / SSPI / certificate | not supported |

HMAC-SHA-256, PBKDF2-HMAC-SHA-256, and MD5 are implemented in-tree on the
existing SHA-256 in `src/system.rs`. Each is pinned to published vectors —
RFC 4231, RFC 6070, RFC 1321, and the full SCRAM exchange from RFC 7677 —
because a silently wrong digest surfaces only as an unexplained authentication
failure, which is painful to diagnose.

MD5 is cryptographically broken. It is implemented only because deployments
still request it, and should never be used for anything else.

## Limits

Understand these before depending on the client:

- **No TLS.** Connections are cleartext, so credentials and row data cross the
  network unprotected. Use a trusted network or a tunnel. Wiring this through the
  optional `openssl-tls` transport is open work.
- **Simple query only.** There is no extended-protocol `Parse`/`Bind`, so values
  cannot be bound as parameters. Untrusted input must not be concatenated into
  SQL text. This is the most important gap to close next.
- **Text-format decoding**, per the table above.
- **One synchronous connection.** Pooling belongs to the host, matching how
  `DatabaseAuthority` is granted per request.

## Testing

Wire compatibility cannot be proven against a mock: the SCRAM exchange, message
framing, and row decoding are only meaningful against a real server. The tests in
`tests/postgres_live.rs` therefore need one, and skip unless
`TETHERSCRIPT_PG_TEST_URL` is set, so the default `cargo test` run stays
hermetic.

```bash
docker run -d --rm --name ts_pg_test \
  -e POSTGRES_PASSWORD=pencil -e POSTGRES_USER=tsuser -e POSTGRES_DB=tsdb \
  -p 55432:5432 postgres:16

docker exec ts_pg_test psql -U tsuser -d tsdb -c \
  "CREATE TABLE users (id int, name text, active bool, score numeric, note text);
   INSERT INTO users VALUES (1,'Riley',true,9.5,NULL),(2,'Ada',false,7.25,'hi');"

TETHERSCRIPT_PG_TEST_URL=127.0.0.1:55432 cargo test --test postgres_live
```

To exercise the legacy `md5` path instead, start the server with
`-e POSTGRES_HOST_AUTH_METHOD=md5 -e POSTGRES_INITDB_ARGS=--auth-host=md5`.
Both paths were verified against PostgreSQL 16.

The crypto primitives have unit coverage that runs unconditionally, so a
regression in SHA-256, HMAC, PBKDF2, or MD5 is caught by a normal `cargo test`
without a database present.
