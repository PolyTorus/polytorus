use actix_web::{get, HttpResponse, Responder};
use crate::wallet::transaction_pool::Pool;

#[get("/transactions")]
async fn transactions() -> impl Responder {
    let pool = Pool::new();
    let transactions = pool.transactions;

    HttpResponse::Ok().json(transactions)
}