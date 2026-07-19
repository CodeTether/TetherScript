# Actix Web: native Rust and tetherscript routes

This standalone project puts two controllers in one Actix Web `App` so their
registration and behavior can be compared directly:

| URL | Controller | Registration |
|---|---|---|
| `GET /rust/country/{code}` | Native Rust + PostgreSQL | `App::route` |
| `GET /tether/country/{code}` | Hot-reloaded tetherscript + PostgreSQL | `ActixPlugin::configure` |

Both routes are real Actix services. Actix still owns the listener, routing,
path matching, request limits, middleware, logging, TLS, and deployment. The
second route delegates only its controller logic to tetherscript.

## Create the isolated database

Configure the PostgreSQL endpoint for your environment:

```text
DATABASE_HOST=<postgres-host>
DATABASE_PORT=5432
DATABASE_USER=<demo-database-admin>
```

Set `PGPASSWORD` through a protected environment mechanism, or set
`PGPASSWORD_FILE` to a protected file containing only the password. Neither
value is committed or logged. Then run:

```bash
cargo run --manifest-path examples/actix-web-demo/Cargo.toml --bin bootstrap
```

The bootstrap creates only `tetherscript_actix_demo` on the configured host, checks
`current_database()` before schema changes, and imports the World Bank 2022 GDP
feed transactionally. It does not connect to or modify `a2a_server`. Both HTTP
routes use the same r2d2 pool, ranked SQL query, and JSON response shape.

## Run it

From the tetherscript repository root:

```bash
cargo run --manifest-path examples/actix-web-demo/Cargo.toml --bin tetherscript-actix-web-demo
```

The server listens on `127.0.0.1:18081`. Test the native handler:

```bash
curl -i http://127.0.0.1:18081/rust/country/USA
```

Expected controller evidence:

```text
x-controller: rust

{"code":"USA","name":"United States","year":2022,"gdp_usd":...}
```

Then test the tetherscript handler:

```bash
curl -i http://127.0.0.1:18081/tether/country/USA
```

Expected controller evidence:

```text
x-controller: tetherscript

{"code":"USA","name":"United States","year":2022,"gdp_usd":...}
```

## Benchmark both routes

Keep the server running, then start the dependency-free Node.js benchmark in a
second terminal:

```bash
node examples/actix-web-demo/benchmark.js 3000 32 3
```

The arguments are requests per route per round, concurrency, and round count.
The benchmark:

1. Uses one HTTP keep-alive pool for each sample.
2. Warms both routes before timing them.
3. Alternates measurement order each round to reduce order bias.
4. Sends the same method and path-parameter workload to both controllers.
5. Reports failures, requests/second, mean latency, min/max latency, and bytes.

This measures controller dispatch through a real Actix server. It is not a
general Rust-versus-tetherscript language benchmark: the tetherscript route also
crosses `web::block`, converts request/response values, and enforces plugin
budgets. Those are intentional production costs of safely offloading a route.

For a longer run:

```bash
node examples/actix-web-demo/benchmark.js 20000 64 5
```

Save raw evidence when comparing changes:

```bash
node examples/actix-web-demo/benchmark.js 10000 32 5 > benchmark-results.json
```

## Native route

The regular route checks out a pooled connection on Actix's blocking pool:

```rust
async fn country(pool: Data<DbPool>, code: Path<String>) -> HttpResponse {
    web::block(move || database::country(&pool, &code)).await
}
```

It is registered in the usual way:

```rust
.route("/rust/country/{code}", web::get().to(rust_route::country))
```

## tetherscript route

The script file is validated at startup and hot-reloaded after valid edits:

```rust
let tether_route = ActixPlugin::from_file(
    "/tether/country/{code}",
    Method::GET,
    "controllers/tether_route.tether",
)
.build()?;
```

The validated route is registered on the same `ServiceConfig` as Rust routes:

```rust
.configure(move |config| tether_route.configure(config))
```

Actix passes a request snapshot to the script's `handle` hook. Available fields
are `method`, `path`, `query`, `headers`, `params`, and `body`. UTF-8 bodies are
strings; other bodies are tetherscript `Bytes` values.

The hook returns an HTTP response map:

```tether
fn handle(request) {
    let body = db.country(request.params.code).unwrap()
    let response = map()
    response.status = 200
    response.headers = map()
    response.headers["content-type"] = "application/json"
    response.body = body
    return response
}
```

Status codes and header names/values are validated before Actix constructs the
response. Script failures and invalid response maps become HTTP 500 responses
with a diagnostic instead of panicking an Actix worker.

## Execution and safety

tetherscript hooks run through `actix_web::web::block`, not on Actix's async
request workers. Each blocking-pool thread caches its own loaded interpreter.
The plugin host enforces its instruction and output budgets, and scripts start
with the sandboxed builtin set. External authority is opt-in.

## Database and Rust service access

Use `host_factory` to grant a Rust capability backed by application state. The
factory is `Send + Sync`; it creates the thread-local `PluginHost` and its `Rc`
capabilities inside the blocking thread:

```rust
let route = ActixPlugin::builder("/users/{id}", Method::GET, source)
    .host_factory({
        let pool = pool.clone();
        move || {
            let mut host = PluginHost::new();
            host.grant("db", Rc::new(DatabaseAuthority::new(pool.clone())));
            host
        }
    })
    .build()?;
```

This demo calls `db.country(request.params.code).unwrap()`. Synchronous
repositories and pools fit the current capability interface directly. Async DB
clients should be placed behind a synchronous repository/queue bridge because
the tetherscript `Authority::invoke` contract is synchronous.

## Project layout

```text
actix-web-demo/
  Cargo.toml
  README.md
  benchmark.js
  controllers/tether_route.tether
  src/main.rs
```