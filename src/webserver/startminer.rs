// Legacy CLI command import removed in Phase 4 - using modular architecture
// use crate::command::cil_startminer::cmd_start_miner_from_api;
use actix_web::{post, web, HttpResponse, Responder};
use serde::Deserialize;

#[derive(Deserialize)]
#[allow(dead_code)]
struct StartMinerRequest {
    host: String,
    port: String,
    bootstrap: Option<String>,
    mining_address: String,
}

#[post("/start-miner")]
pub async fn start_miner(_req: web::Json<StartMinerRequest>) -> impl Responder {
    HttpResponse::NotImplemented().body("Legacy miner has been removed. Use 'polytorus modular mine' commands instead.")
}
