use super::global::PostPoolJson;
use crate::app::global::{CHAIN, POOL, SERVER, WALLET};
use actix_web::{post, web, HttpResponse, Responder};

#[post("/transact")]
async fn transact(data: web::Json<PostPoolJson>) -> impl Responder {
    let (recipient, amount) = (data.recipient.clone(), data.amount.clone());
    let chain = CHAIN.lock().await.clone();
    let mut pool = POOL.lock().await.clone();


    let transaction = {
        let mut wallet = WALLET.lock().await.clone();
        wallet.create_transaction(recipient, amount, chain.clone(), &mut pool).unwrap()
    };
    
    if let Some(server) = SERVER.lock().await.as_ref() {
        server.broadcast_transaction(transaction.clone()).await;
    }

    HttpResponse::Ok().json(transaction)
}
