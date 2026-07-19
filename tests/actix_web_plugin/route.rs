use actix_web::{http::Method, test as awtest, App};

use tetherscript::actix_web::ActixPlugin;

const SOURCE: &str = r#"
fn handle(request) {
    let response = map()
    response.status = 201
    response.headers = map()
    response.headers["x-method"] = request.method
    response.body = request.params.id + ":" + request.query + ":" + request.body
    return response
}
"#;

#[actix_web::test]
async fn routes_requests_through_tetherscript() {
    let plugin = ActixPlugin::builder("/items/{id}", Method::POST, SOURCE)
        .build()
        .unwrap();
    let app = awtest::init_service(App::new().configure(|config| plugin.configure(config))).await;
    let request = awtest::TestRequest::post()
        .uri("/items/42?fresh=true")
        .set_payload("payload")
        .to_request();
    let response = awtest::call_service(&app, request).await;
    assert_eq!(response.status(), 201);
    assert_eq!(response.headers().get("x-method").unwrap(), "POST");
    assert_eq!(awtest::read_body(response).await, "42:fresh=true:payload");
}

#[test]
fn rejects_missing_hook_at_startup() {
    let error = match ActixPlugin::builder("/missing", Method::GET, "let value = 1").build() {
        Ok(_) => panic!("missing hook should fail"),
        Err(error) => error,
    };
    assert!(error.to_string().contains("missing hook `handle`"));
}
