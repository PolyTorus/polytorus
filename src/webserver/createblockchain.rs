use actix_web::{post, web, HttpResponse, Responder};
use serde::Deserialize;

use crate::command::cli_createblockchain::cmd_create_blockchain_from_api;

#[derive(Deserialize)]
struct Address {
    address: String,
}

#[post("/create_blockchain")]
pub async fn create_blockchain(req: web::Json<Address>) -> impl Responder {
    let req_data = req.into_inner();
    let address = req_data.address.as_str();

    match cmd_create_blockchain_from_api(address) {
        Ok(()) => {
            println!("create blockchain address: {:?}", address);
            HttpResponse::Ok().body("Blockchain created")
        }
        Err(e) => {
            HttpResponse::InternalServerError().body(format!("Blockchain creation failed: {}", e))
        }
    }
}
