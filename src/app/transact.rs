use super::global::PostPoolJson;
use crate::{
    app::global::{CHAIN, POOL, SERVER, WALLET},
    wallet::wallets::Wallet,
};
use actix_web::{post, web, HttpResponse, Responder};
use tokio::sync::MutexGuard;

#[post("/transact")]
async fn transact(data: web::Json<PostPoolJson>) -> impl Responder {
    let (recipient, amount) = (data.recipient.clone(), data.amount);

    let transaction = {
        let chain = CHAIN.lock().await;
        let mut pool = POOL.lock().await;
        let mut wallet = WALLET.lock().await;

        match wallet.create_transaction(recipient, amount, &chain, &mut pool) {
            Ok(tx) => tx,
            Err(e) => {
                return HttpResponse::BadRequest()
                    .json(format!("トランザクションの作成に失敗しました: {}", e))
            }
        }
    };

    {
        let mut pool = POOL.lock().await;
        pool.update_or_add_transaction(transaction.clone());
    }

    {
        let mut server = SERVER.lock().await;
        if let Some(server_instance) = server.as_mut() {
            if let Err(e) = server_instance.broadcast_transaction(transaction).await {
                eprintln!("BroadCast Error: {}", e);
            }
        }
    }

    let valid_transactions = {
        let pool = POOL.lock().await;
        pool.valid_transactions()
    };

    HttpResponse::Ok().json(valid_transactions)
}
