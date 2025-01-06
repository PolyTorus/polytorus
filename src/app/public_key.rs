use crate::app::global::WALLET;
use actix_web::{get, HttpResponse, Responder};

#[get("/public-key")]
async fn public_key() -> impl Responder {
    HttpResponse::Ok().json(WALLET.lock().await.public_key)
}
