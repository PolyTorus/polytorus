use actix_web::{get, HttpResponse, Responder};
use crate::app::global::WALLET;

#[get("/public-key")]
async fn public_key() -> impl Responder {
    HttpResponse::Ok().json(WALLET.public_key)
}
