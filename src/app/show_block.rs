use crate::blockchain::chain::Chain;
use actix_web::{get, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

// block struct to json
#[derive(Serialize, Deserialize)]
struct BlockJson {
    timestamp: u64,
    last_hash: String,
    hash: String,
    data: String,
}


#[get("/block")]
async fn block() -> impl Responder {
    let mut chain = Chain::new();
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
