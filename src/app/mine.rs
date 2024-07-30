use super::global::{CHAIN, BlockJson, PostBlockJson};
use actix_web::{post, HttpResponse, Responder, web};

// /mine endpoint
#[post("/mine")]
async fn mine(data: web::Json<PostBlockJson>) -> impl Responder {
    let mut chain = CHAIN.lock().unwrap();
    let block = chain.add_block(data.data.clone());

    let block_json = BlockJson {
        timestamp: block.timestamp,
        last_hash: block.last_hash.clone(),
        hash: block.hash.clone(),
        data: block.data.clone(),
    };

    let json = serde_json::to_string(&block_json).unwrap();

    HttpResponse::Ok().body(json)
}
