use super::global::{CHAIN, BlockJson};
use actix_web::{get, HttpResponse, Responder};

#[get("/block")]
async fn block() -> impl Responder {
    let mut chain = CHAIN.lock().unwrap();
    let block = chain.add_block("block".to_string());

    let block_json = BlockJson {
        timestamp: block.timestamp,
        last_hash: block.last_hash.clone(),
        hash: block.hash.clone(),
        data: block.data.clone(),
    };

    let json = serde_json::to_string(&block_json).unwrap();

    HttpResponse::Ok().body(json)
}
