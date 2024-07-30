use super::global::{CHAIN, BlockJson};
use actix_web::{post, HttpResponse, Responder};

// /mine endpoint
#[post("/mine")]
async fn mine() -> impl Responder {
    let mut chain = CHAIN.lock().unwrap();
    let block = chain.add_block("block".to_string());

    let block_json = BlockJson {
        timestamp: block.timestamp,
        last_hash: block.last_hash.clone(),
        hash: block.hash.clone(),
        data: block.data.clone(),
    };

    let json = serde_json::to_string(&block_json).unwrap();

    // redirect to /block
    HttpResponse::Found()
        .append_header(("location", "/block"))
        .body(json)
}