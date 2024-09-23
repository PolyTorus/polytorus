use crate::app::global::POOL;
use actix_web::{get, HttpResponse, Responder};

#[get("/transactions")]
async fn transactions() -> impl Responder {
    let pool = POOL.lock().unwrap();
    let transactions = serde_json::to_string(&pool.transactions).unwrap();

    HttpResponse::Ok().json(transactions)
}
