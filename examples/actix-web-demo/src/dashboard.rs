//! Documentation and live route-comparison dashboard.

use actix_web::{http::header::ContentType, HttpResponse};

const PAGE: &str = include_str!("../dashboard.html");

pub(super) async fn page() -> HttpResponse {
    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(PAGE)
}
