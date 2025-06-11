// Legacy command removed - print chain functionality not available in modern CLI
use actix_web::{post, HttpResponse, Responder};

#[post("/print-chain")]
pub async fn print_chain() -> impl Responder {
    HttpResponse::NotImplemented()
        .body("Print chain functionality has been removed in modern architecture")
}
