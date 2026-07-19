//! Native Rust controller for the shared database workload.

use actix_web::{web, HttpResponse};

use crate::database;
use crate::db_pool::DbPool;

pub async fn country(pool: web::Data<DbPool>, code: web::Path<String>) -> HttpResponse {
    let pool = pool.get_ref().clone();
    let code = code.into_inner();
    match web::block(move || database::country(&pool, &code)).await {
        Ok(Ok(Some(record))) => HttpResponse::Ok()
            .insert_header(("x-controller", "rust"))
            .json(record),
        Ok(Ok(None)) => HttpResponse::NotFound()
            .insert_header(("x-controller", "rust"))
            .body("country not found"),
        Ok(Err(error)) => HttpResponse::InternalServerError()
            .insert_header(("x-controller", "rust"))
            .body(error),
        Err(error) => HttpResponse::InternalServerError()
            .insert_header(("x-controller", "rust"))
            .body(error.to_string()),
    }
}
