//! Side-by-side native Rust and tetherscript Actix Web routes.
//!
//! Run from the repository root with:
//! `cargo run --manifest-path examples/actix-web-demo/Cargo.toml --bin tetherscript-actix-web-demo`.

mod dashboard;
mod database;
mod db_authority;
mod db_bind;
mod db_config;
mod db_decode;
mod db_decode_numeric;
mod db_pool;
mod db_query;
mod db_row;
mod rust_route;
mod tether_route;

use actix_web::{web, App, HttpServer};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let pool = db_pool::create()
        .await
        .expect("database pool should connect");
    let tether_route = tether_route::build(pool.clone(), tokio::runtime::Handle::current());
    let pool = web::Data::new(pool);

    println!("dashboard:         http://127.0.0.1:18081/");
    println!("regular route:     http://127.0.0.1:18081/rust/country/USA");
    println!("tetherscript route: http://127.0.0.1:18081/tether/country/USA");
    HttpServer::new(move || {
        let tether_route = tether_route.clone();
        App::new()
            .app_data(pool.clone())
            // Interactive documentation and browser benchmark.
            .route("/", web::get().to(dashboard::page))
            // Route one: ordinary Actix Web registration.
            .route("/rust/country/{code}", web::get().to(rust_route::country))
            // Route two: the same ServiceConfig, delegated to tetherscript.
            .configure(move |config| tether_route.configure(config))
    })
    .bind(("127.0.0.1", 18081))?
    .run()
    .await
}
