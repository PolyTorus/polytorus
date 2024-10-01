use super::global::PostPoolJson;
use crate::{app::global::{CHAIN, POOL, SERVER, WALLET}, wallet::wallets::Wallet};
use actix_web::{post, web, HttpResponse, Responder};
use tokio::sync::MutexGuard;

#[post("/transact")]
async fn transact(data: web::Json<PostPoolJson>) -> impl Responder {
    let (recipient, amount) = (data.recipient.clone(), data.amount.clone());

    let (chain, mut pool, mut wallet, mut server) = tokio::join!(
        CHAIN.lock(),
        POOL.lock(),
        WALLET.lock(),
        SERVER.lock(),
    );

    let transaction = match wallet.create_transaction(recipient, amount, &chain, &mut pool) {
        Ok(tx) => tx,
        Err(e) => return HttpResponse::BadRequest().json(format!("Failed to create transaction: {}", e)),
    };
    
    if let Some(server_instance) = server.as_mut() {
        server_instance.broadcast_transaction(transaction.clone()).await;
    }

    HttpResponse::Ok().json(transaction)
}
