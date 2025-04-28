use crate::command::cli::cmd_print_chain;
use actix_web::{post, HttpResponse, Responder};

#[post("/print-chain")]
pub async fn print_chain() -> impl Responder {
    match cmd_print_chain() {
        Ok(()) => HttpResponse::Ok().body("Complete print chain"),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}
