use crate::command::cil_listaddresses::cmd_list_address;
use actix_web::{post, HttpResponse, Responder};

#[post("/list-addresses")]
pub async fn list_addresses() -> impl Responder {
    match cmd_list_address() {
        Ok(()) => HttpResponse::Ok().body("Complete list adresses"),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}
