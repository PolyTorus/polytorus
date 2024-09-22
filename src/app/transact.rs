use super::global::PostPoolJson;
use actix_web::{post, HttpResponse, Responder, web};
use crate::app::global::{WALLET, POOL, SERVER, CHAIN};

#[post("/transact")]
async fn transact(data: web::Json<PostPoolJson>) -> impl Responder {
    let (recipient, amount) = (data.receipient.clone(), data.amount.clone());
    let chain = CHAIN.lock().unwrap().clone();
    let pool = POOL.lock().unwrap();
    let transaction = WALLET.lock().unwrap().create_transaction(recipient, amount, chain, &mut pool.clone());
    SERVER.broadcast_transaction(transaction.clone().unwrap()).await;

    HttpResponse::Ok().json(transaction)
}
