use super::global::PostPoolJson;
use crate::{app::global::{CHAIN, POOL, SERVER, WALLET}, wallet::wallets::Wallet};
use actix_web::{post, web, HttpResponse, Responder};

#[post("/transact")]
async fn transact(data: web::Json<PostPoolJson>) -> impl Responder {
    let (recipient, amount) = (data.recipient.clone(), data.amount.clone());
    let chain = CHAIN.lock().await.clone();
    let mut pool = POOL.lock().await.clone();


    let transaction = {
        let mut wallet: Wallet = WALLET.lock().await.clone();
        match wallet.create_transaction(recipient, amount, chain.clone(), &mut pool) {
            Ok(tx) => tx,
            Err(e) => return HttpResponse::BadRequest().json(format!("Failed to create transaction: {}", e)),
        }
    };
    
    if let Some(server) = SERVER.lock().await.as_ref() {
        server.broadcast_transaction(transaction.clone()).await;
    }

    HttpResponse::Ok().json(transaction)
}
