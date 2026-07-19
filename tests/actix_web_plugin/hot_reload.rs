use std::fs;
use std::time::{Duration, SystemTime};

use actix_web::{http::Method, test as awtest, App};
use tetherscript::actix_web::ActixPlugin;

fn source(body: &str) -> String {
    format!(
        "fn handle(request) {{\n  let response = map()\n  response.body = \"{body}\"\n  return response\n}}"
    )
}

#[actix_web::test]
async fn reloads_valid_edits_and_keeps_last_good_source() {
    let nonce = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("tetherscript-actix-{nonce}.tether"));
    fs::write(&path, source("generation-one")).unwrap();
    let plugin = ActixPlugin::from_file("/reload", Method::GET, &path)
        .build()
        .unwrap();
    let app = awtest::init_service(App::new().configure(|config| plugin.configure(config))).await;

    let request = awtest::TestRequest::get().uri("/reload").to_request();
    let response = awtest::call_and_read_body(&app, request).await;
    assert_eq!(response, "generation-one");

    std::thread::sleep(Duration::from_millis(20));
    fs::write(&path, source("generation-two")).unwrap();
    let request = awtest::TestRequest::get().uri("/reload").to_request();
    let response = awtest::call_and_read_body(&app, request).await;
    assert_eq!(response, "generation-two");

    std::thread::sleep(Duration::from_millis(20));
    fs::write(&path, "fn handle(").unwrap();
    let request = awtest::TestRequest::get().uri("/reload").to_request();
    let response = awtest::call_and_read_body(&app, request).await;
    assert_eq!(response, "generation-two");
    assert!(plugin.reload_error().is_some());
    fs::remove_file(path).unwrap();
}
