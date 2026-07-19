//! Live Actix Web server with a tetherscript-backed route.

use actix_web::{http::Method, App, HttpServer};
use tetherscript::actix_web::ActixPlugin;

const HOME: &str = r#"
fn handle(request) {
    let response = map()
    response.headers = map()
    response.headers["content-type"] = "text/html; charset=utf-8"
    response.body = "<main><h1>Actix Web + tetherscript</h1><p>The GET / route is running in tetherscript.</p><form method=\"post\" action=\"/hello/Riley\"><input name=\"message\" value=\"from-browser\"><button>Call tetherscript</button></form></main>"
    return response
}
"#;

const HELLO: &str = r#"
fn handle(request) {
    let response = map()
    response.status = 202
    response.headers = map()
    response.headers["x-powered-by"] = "tetherscript"
    response.body = "hello " + request.params.name + ":" + request.body
    return response
}
"#;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let home = ActixPlugin::builder("/", Method::GET, HOME)
        .plugin_name("actix-home")
        .build()
        .expect("home route should load");
    let hello = ActixPlugin::builder("/hello/{name}", Method::POST, HELLO)
        .plugin_name("live-actix-example")
        .build()
        .expect("tetherscript route should load");
    let address = ("127.0.0.1", 18080);
    println!("listening on http://{}:{}", address.0, address.1);
    HttpServer::new(move || {
        let home = home.clone();
        let hello = hello.clone();
        App::new().configure(move |config| {
            home.configure(config);
            hello.configure(config);
        })
    })
    .bind(address)?
    .run()
    .await
}
