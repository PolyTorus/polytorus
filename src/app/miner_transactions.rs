use crate::app::global::MINER;
use actix_web::{get, HttpResponse, Responder};

#[get("/miner-transactions")]
async fn miner_transactions() -> impl Responder {
    let block = {
        let mut miner = MINER.lock().unwrap();
        miner.mine().await
    };
    println!("Block mined: {:?}", block.clone());

    HttpResponse::Ok().json(block)
}
