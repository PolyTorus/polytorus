use crate::command::cli_reindex::cmd_reindex;
use actix_web::{post, HttpResponse, Responder};

#[post("/reindex")]
pub async fn reindex() -> impl Responder {
    match cmd_reindex() {
        Ok(()) => HttpResponse::Ok().body("Complete reindex"),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}
