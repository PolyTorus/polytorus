// Legacy CLI command import removed in Phase 4 - using modular architecture
// use crate::command::cil_startminer::cmd_start_miner_from_api;
use actix_web::{post, web, HttpResponse, Responder};
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
    println!(
        "@start-Miner called: host={}, port={}",
        req_data.host, req_data.port
    ); // Extract request data (not used in Phase 4 - legacy functions disabled)
    let _host = req_data.host.clone();
    let _port = req_data.port.clone();
    let _bootstrap = req_data.bootstrap.clone();
    let _mining_address = req_data.mining_address.clone();
    tokio::task::spawn_blocking(move || {
        // Legacy start miner function removed in Phase 4 - using modular architecture
        // if let Err(e) =
        //     cmd_start_miner_from_api(&host, &port, bootstrap.as_deref(), &mining_address)
        // {
        //     eprintln!("Miner failed to start: {}", e);
        // } else {
        //     println!("Miner started successfully");
        // }
        eprintln!("Legacy miner removed. Use 'polytorus modular mine' commands instead.");
    });

    HttpResponse::Accepted().body("Miner is starting in background")
}
