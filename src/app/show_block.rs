use super::global::CHAIN;
use actix_web::{get, HttpResponse, Responder};

#[get("/block")]
async fn block() -> impl Responder {
    let chain = CHAIN.lock().unwrap();
    let block = chain.clone();

    let json = serde_json::to_string(&block).unwrap();

    HttpResponse::Ok().body(json)
}
