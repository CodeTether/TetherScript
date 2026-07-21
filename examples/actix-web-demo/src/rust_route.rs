//! Native Rust controller for the shared database workload.

use actix_web::{web, HttpResponse};

use crate::database;
use crate::db_pool::DbPool;

pub async fn country(pool: web::Data<DbPool>, code: web::Path<String>) -> HttpResponse {
    let code = code.into_inner();
    match database::country(pool.get_ref(), &code).await {
        Ok(Some(record)) => HttpResponse::Ok()
            .insert_header(("x-controller", "rust"))
            .json(record),
        Ok(None) => HttpResponse::NotFound()
            .insert_header(("x-controller", "rust"))
            .body("country not found"),
        Err(error) => HttpResponse::InternalServerError()
            .insert_header(("x-controller", "rust"))
            .body(error),
    }
}
