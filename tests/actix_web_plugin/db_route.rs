use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use actix_web::{http::Method, test as awtest, App};

use tetherscript::actix_web::ActixPlugin;
use tetherscript::plugin::PluginHost;

use super::authority::DbAuthority;

const SOURCE: &str = r#"
fn handle(request) {
    let response = map()
    response.body = db.lookup(request.params.id).unwrap()
    return response
}
"#;

#[actix_web::test]
async fn injects_rust_database_capability() {
    let calls = Arc::new(AtomicUsize::new(0));
    let factory_calls = calls.clone();
    let plugin = ActixPlugin::builder("/records/{id}", Method::GET, SOURCE)
        .host_factory(move || {
            let mut host = PluginHost::new();
            host.grant("db", Rc::new(DbAuthority::new(factory_calls.clone())));
            host
        })
        .build()
        .unwrap();
    let app = awtest::init_service(App::new().configure(|config| plugin.configure(config))).await;
    let request = awtest::TestRequest::get().uri("/records/7").to_request();
    let response = awtest::call_service(&app, request).await;
    assert!(response.status().is_success());
    assert_eq!(awtest::read_body(response).await, "record:7");
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}
