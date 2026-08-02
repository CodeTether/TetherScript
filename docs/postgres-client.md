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
use tetherscript::value::Value;

let mut connection = Connection::connect(&Config {
    host: "127.0.0.1".into(),
    port: 5432,
    user: "tsuser".into(),
    password: "pencil".into(),
    database: "tsdb".into(),
})?;

// `query` binds parameters through the extended protocol. Use it for anything
// that came from outside the program.
let rows = connection.query(
    "SELECT id, name FROM users WHERE id = $1",
    &[Value::Int(1)],
)?;
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

## Reaching it from a script

### From the CLI

`--grant-db` connects the native client and grants it as `db`, so a script run from
the command line reaches SQL with no Rust host involved:

```bash
tetherscript run --grant-db postgres://user:pass@localhost:5432/app app.tether
```

The URL is parsed and the connection established *before* the script starts, so a
typo or an unreachable database fails immediately rather than at the first query.
Unlike `--grant-fs`, this is never implied by `--access-mode full`: a connection
string carries credentials that cannot be inferred from the environment, so the
grant is always explicit. Without it, `db` is undefined and the script fails
closed.

### From a Rust host

`PostgresHandler` implements `QueryHandler`, so a host grants it as the `db`
capability and a `.tether` script queries through it. Scripts have no ambient
database access: `db` is undefined unless it was granted.

```rust,no_run
use std::rc::Rc;
use tetherscript::database::DatabaseAuthority;
use tetherscript::plugin::PluginHost;
use tetherscript::postgres::{Config, PostgresHandler};

let handler = PostgresHandler::connect(&config)?;
let mut host = PluginHost::new();
host.grant("db", Rc::new(DatabaseAuthority::new(handler)));
```

The script then passes parameters as a separate list, never as SQL text:

```tether
fn find_user(id) {
    let rows = db.query("SELECT id, name FROM users WHERE id = $1", [id])?
    if rows.len() == 0 {
        return Err("no user with id " + str(id))
    }
    return Ok(rows[0])
}
```

Because the value is bound rather than spliced, a quoting attack is inert:

```tether
// Matches no row. It does not terminate the statement, and the table survives.
let rows = db.query("SELECT id FROM users WHERE name = $1",
                    ["Ada'; DROP TABLE users; --"])?
```

That behaviour is asserted, not assumed — see `a_bound_parameter_cannot_terminate_the_statement`
in `tests/postgres_live.rs` and `bound_parameters_cannot_inject_sql` in
`tests/db_capability_live.rs`.

## Pooling

`PostgresHandler` serves queries from a small pool, because `http_serve` runs a
single-threaded accept loop and one connection would put every request behind the
slowest statement. Connections are opened lazily up to the limit, so a script that
never queries pays for nothing.

```tether
println(str(db.pool_size()?))   // connections currently held
```

A connection whose exchange failed mid-stream is discarded rather than reused: it
may still have unread bytes queued, which would misalign every later reply. A
server-side SQL error is different — the reply was drained through
`ReadyForQuery` — so that connection returns to the pool.

## Transactions

`begin` pins one connection until `commit` or `rollback`. Pinning is required, not
an optimization: without it a pooled handler could send the statements to a
different connection, where they would commit independently and survive a
rollback.

```tether
db.begin()?
let created = store.create(code, url, expires_at)
if created.is_err() {
    db.rollback()?
    return error_response()
}
db.commit()?
```

Three deliberate refusals, each an error rather than a silent no-op:

- **Nested `begin`** — a caller that believes it has an inner scope would otherwise
  have its rollback discard the outer work too.
- **`commit` or `rollback` with nothing open** — usually a control-flow bug.
- **Either, on an adapter that has not opted in.** `QueryHandler`'s default
  implementations return an error, so a handler cannot appear to honour a
  transaction it is ignoring.

Runnable end-to-end example:

```bash
TETHERSCRIPT_PG_URL=127.0.0.1:55432 cargo run --example db_capability
```

## Limits

Understand these before depending on the client:

- **No TLS.** Connections are cleartext, so credentials and row data cross the
  network unprotected. Use a trusted network or a tunnel. Wiring this through the
  optional `openssl-tls` transport is open work.
- **Parameters bind as text.** `query` uses the extended protocol, so values never
  enter the SQL string, but the server infers each type rather than being told it.
  Supported parameter types are str, int, float, bool, and nil. `simple_query`
  accepts no parameters at all, so untrusted input belongs in `query`.
- **No binary format**, in either direction.
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

The extended-protocol message bytes are also asserted unconditionally in
`src/postgres/extended_tests.rs`, covering the `Parse` layout, NULL as a -1
length, text-format value framing, and rejection of unbindable parameter types by
position. Framing bugs there would otherwise surface as an opaque server
complaint or a hung read.