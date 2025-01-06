use crate::app::global::MINER;
use crate::app::minner::Minner;
use actix_web::{get, HttpResponse, Responder};

#[get("/miner-transactions")]
async fn miner_transactions() -> impl Responder {
    let block = {
        let mut miner: Minner = MINER.lock().await.clone().expect("Failed to lock MINER");
        miner.mine().await
    };
    println!("Block mined: {:?}", block.clone());

    HttpResponse::Ok().json(block)
}
