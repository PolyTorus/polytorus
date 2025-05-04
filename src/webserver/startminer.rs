use actix_web::{post, web, HttpResponse, Responder};
use crate::command::cil_startminer::cmd_start_miner_from_api;
use serde::Deserialize;

#[derive(Deserialize)]
struct StartMinerRequest {
    host: String,
    port: String,
    bootstrap: Option<String>,
    mining_address: String,
}

#[post("/start-miner")]
pub async fn start_miner(req: web::Json<StartMinerRequest>) -> impl Responder {
    let req_data = req.into_inner();
    println!("@start-Miner called: host={}, port={}", req_data.host, req_data.port);

    let host = req_data.host.clone();
    let port = req_data.port.clone();
    let bootstrap = req_data.bootstrap.clone();
    let mining_address = req_data.mining_address.clone();

    tokio::task::spawn_blocking(move || {
        if let Err(e) = cmd_start_miner_from_api(&host, &port, bootstrap.as_deref(), &mining_address) {
            eprintln!("Miner failed to start: {}", e);
        } else {
            println!("Miner started successfully");
        }
    });

    HttpResponse::Accepted().body("Miner is starting in background")
}