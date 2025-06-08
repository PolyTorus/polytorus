// Legacy CLI command import removed in Phase 4 - using modular architecture
// use crate::command::cil_listaddresses::cmd_list_address;
use crate::command::cli::cmd_list_address;
use actix_web::{post, HttpResponse, Responder};

#[post("/list-addresses")]
pub async fn list_addresses() -> impl Responder {
    match cmd_list_address() {
        Ok(()) => HttpResponse::Ok().body("Complete list addresses"),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}
