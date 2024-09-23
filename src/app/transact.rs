use super::global::PostPoolJson;
use crate::app::global::{CHAIN, POOL, SERVER, WALLET};
use actix_web::{post, web, HttpResponse, Responder};

#[post("/transact")]
async fn transact(data: web::Json<PostPoolJson>) -> impl Responder {
    let (recipient, amount) = (data.receipient.clone(), data.amount.clone());
    let chain = CHAIN.lock().unwrap().clone();
    let pool = POOL.lock().unwrap();
    let transaction =
        WALLET
            .lock()
            .unwrap()
            .create_transaction(recipient, amount, chain, &mut pool.clone());
    SERVER
        .broadcast_transaction(transaction.clone().unwrap())
        .await;

    HttpResponse::Ok().json(transaction)
}
