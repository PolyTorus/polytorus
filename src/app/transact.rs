use super::global::PostPoolJson;
use actix_web::{post, HttpResponse, Responder, web};
use crate::app::global::{WALLET, POOL, SERVER};

#[post("/transact")]
async fn transact(data: web::Json<PostPoolJson>) -> impl Responder {
    let (recipient, amount) = (data.receipient.clone(), data.amount.clone());
    let transaction = WALLET.create_transaction(recipient, amount, &mut POOL.lock().unwrap().clone()).unwrap();
    SERVER.broadcast_transaction(transaction.clone()).await;

    HttpResponse::Ok().json(transaction)
}
