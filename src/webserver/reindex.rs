// Legacy CLI command import removed in Phase 4 - using modular architecture
// use crate::command::cil_reindex::cmd_reindex;
use crate::command::cli::cmd_reindex;
use actix_web::{post, HttpResponse, Responder};

#[post("/reindex")]
pub async fn reindex() -> impl Responder {
    match cmd_reindex() {
        Ok(_result) => HttpResponse::Ok().body("Complete reindex"),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}
